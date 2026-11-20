// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! The objects a VST3 host hands to a plug-in: the host itself, memory
//! streams for saving settings, parameter change lists, the handler the
//! plug-in's window reports edits to, the frame it asks to resize, and
//! messages with attribute lists (plug-ins send these between their two
//! halves).
//!
//! Every method here is called by plug-in code through a COM vtable, so its
//! pointer arguments come from the plug-in. The VST3 interface contract says
//! what each pointer must be; each `// SAFETY:` comment below names the part
//! of that contract the code relies on. Where the contract allows a null
//! pointer, or a plug-in is known to pass one anyway, the code checks first
//! and returns `kInvalidArgument` instead of crashing the host process.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::ptr;
use std::sync::{Arc, Mutex};

use vst3::Steinberg::Vst::{
    Event, IAttributeList, IAttributeListTrait, IComponentHandler, IComponentHandlerTrait,
    IEventList, IEventListTrait, IHostApplication, IHostApplicationTrait, IMessage, IMessageTrait,
    IParamValueQueue, IParamValueQueueTrait, IParameterChanges, IParameterChangesTrait, ParamID,
    ParamValue, String128, TChar,
};
use vst3::Steinberg::{
    int32, int64, kInvalidArgument, kNoInterface, kResultFalse, kResultOk, kResultTrue, tresult,
    uint32, FIDString, IBStream, IBStreamTrait, IPlugFrame, IPlugFrameTrait, IPlugView, ViewRect,
    TUID,
};
use vst3::{Class, ComWrapper, Interface};

pub fn copy_wstring(src: &str, dst: &mut [TChar]) {
    let mut len = 0;
    for (s, d) in src.encode_utf16().zip(dst.iter_mut()) {
        *d = s as TChar;
        len += 1;
    }
    if len < dst.len() {
        dst[len] = 0;
    } else if let Some(last) = dst.last_mut() {
        *last = 0;
    }
}

pub fn from_wstring(src: &[TChar]) -> String {
    let len = src.iter().position(|c| *c == 0).unwrap_or(src.len());
    String::from_utf16_lossy(&src[..len].iter().map(|c| *c as u16).collect::<Vec<_>>())
}

pub fn from_cstring(src: &[std::ffi::c_char]) -> String {
    let bytes: Vec<u8> = src
        .iter()
        .take_while(|c| **c != 0)
        .map(|c| *c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn same_iid<I: Interface>(tuid: *const TUID) -> bool {
    if tuid.is_null() {
        return false;
    }
    // SAFETY: not null (checked above), and VST3 passes class and interface
    // ids as pointers to a 16-byte TUID, which has the same size and
    // alignment as [u8; 16]. The bytes are only read, and only for this call.
    let bytes = unsafe { &*(tuid as *const [u8; 16]) };
    *bytes == I::IID
}

/// The host, as plug-ins see it.
pub struct HostApp;

impl Class for HostApp {
    type Interfaces = (IHostApplication,);
}

impl IHostApplicationTrait for HostApp {
    unsafe fn getName(&self, name: *mut String128) -> tresult {
        if name.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: not null (checked above); IHostApplication::getName hands
        // a caller-owned String128 (128 UTF-16 units) for the host to fill,
        // and copy_wstring never writes past its length.
        copy_wstring("Altered Audio Editor", unsafe { &mut *name });
        kResultOk
    }

    unsafe fn createInstance(
        &self,
        cid: *mut TUID,
        iid: *mut TUID,
        obj: *mut *mut c_void,
    ) -> tresult {
        if obj.is_null() {
            return kInvalidArgument;
        }
        // The new object's reference passes to the plug-in through `obj`
        // (into_raw gives up our count), as createInstance requires.
        let made = if same_iid::<IMessage>(cid) && same_iid::<IMessage>(iid) {
            ComWrapper::new(Message::default())
                .to_com_ptr::<IMessage>()
                .map(|p| p.into_raw() as *mut c_void)
        } else if same_iid::<IAttributeList>(cid) && same_iid::<IAttributeList>(iid) {
            ComWrapper::new(Attributes::default())
                .to_com_ptr::<IAttributeList>()
                .map(|p| p.into_raw() as *mut c_void)
        } else {
            None
        };
        // SAFETY: not null (checked above); createInstance's `obj` is the
        // caller's out-pointer slot, valid for one pointer-sized write.
        unsafe { *obj = made.unwrap_or(ptr::null_mut()) };
        if made.is_some() {
            kResultOk
        } else {
            kNoInterface
        }
    }
}

/// A growable byte stream in memory.
#[derive(Default)]
pub struct MemStream {
    pub data: RefCell<Vec<u8>>,
    pos: Cell<usize>,
}

impl MemStream {
    pub fn with(data: Vec<u8>) -> Self {
        Self {
            data: RefCell::new(data),
            pos: Cell::new(0),
        }
    }
}

impl Class for MemStream {
    type Interfaces = (IBStream,);
}

impl IBStreamTrait for MemStream {
    unsafe fn read(&self, buffer: *mut c_void, num: int32, read: *mut int32) -> tresult {
        let data = self.data.borrow();
        let at = self.pos.get().min(data.len());
        let n = (num.max(0) as usize).min(data.len() - at);
        if n > 0 && buffer.is_null() {
            return kInvalidArgument;
        }
        if n > 0 {
            // SAFETY: IBStream::read gives a caller buffer of at least `num`
            // bytes; n <= num, the source range at..at+n is inside `data`
            // (n is clamped to what is left), buffer is not null (checked
            // above), and our Vec cannot overlap memory the plug-in owns.
            unsafe { ptr::copy_nonoverlapping(data[at..].as_ptr(), buffer as *mut u8, n) };
        }
        self.pos.set(at + n);
        if !read.is_null() {
            // SAFETY: not null; `read` is an optional caller-owned int32.
            unsafe { *read = n as int32 };
        }
        kResultOk
    }

    unsafe fn write(&self, buffer: *mut c_void, num: int32, written: *mut int32) -> tresult {
        let n = num.max(0) as usize;
        if n > 0 && buffer.is_null() {
            return kInvalidArgument;
        }
        let mut data = self.data.borrow_mut();
        let at = self.pos.get();
        if data.len() < at + n {
            data.resize(at + n, 0);
        }
        if n > 0 {
            // SAFETY: IBStream::write gives `num` readable bytes at `buffer`
            // (not null, checked above); `data` was just grown to at least
            // at + n bytes, so the destination range is in bounds, and the
            // caller's buffer cannot be our own Vec.
            unsafe { ptr::copy_nonoverlapping(buffer as *const u8, data[at..].as_mut_ptr(), n) };
        }
        self.pos.set(at + n);
        if !written.is_null() {
            // SAFETY: not null; `written` is an optional caller-owned int32.
            unsafe { *written = n as int32 };
        }
        kResultOk
    }

    unsafe fn seek(&self, pos: int64, mode: int32, result: *mut int64) -> tresult {
        let len = self.data.borrow().len() as i64;
        let base = match mode {
            0 => 0,
            1 => self.pos.get() as i64,
            2 => len,
            _ => return kInvalidArgument,
        };
        let to = (base + pos).max(0);
        self.pos.set(to as usize);
        if !result.is_null() {
            // SAFETY: not null; `result` is an optional caller-owned int64.
            unsafe { *result = to };
        }
        kResultOk
    }

    unsafe fn tell(&self, pos: *mut int64) -> tresult {
        if !pos.is_null() {
            // SAFETY: not null; `pos` is a caller-owned int64.
            unsafe { *pos = self.pos.get() as int64 };
        }
        kResultOk
    }
}

/// One parameter's changes within a block.
pub struct ParamQueue {
    id: ParamID,
    points: RefCell<Vec<(int32, ParamValue)>>,
}

impl Class for ParamQueue {
    type Interfaces = (IParamValueQueue,);
}

impl IParamValueQueueTrait for ParamQueue {
    unsafe fn getParameterId(&self) -> ParamID {
        self.id
    }

    unsafe fn getPointCount(&self) -> int32 {
        self.points.borrow().len() as int32
    }

    unsafe fn getPoint(&self, index: int32, offset: *mut int32, value: *mut ParamValue) -> tresult {
        if offset.is_null() || value.is_null() {
            return kInvalidArgument;
        }
        match self.points.borrow().get(index.max(0) as usize) {
            Some((o, v)) => {
                // SAFETY: both not null (checked above); getPoint's offset
                // and value are caller-owned out-parameters.
                unsafe {
                    *offset = *o;
                    *value = *v;
                }
                kResultTrue
            }
            None => kResultFalse,
        }
    }

    unsafe fn addPoint(&self, offset: int32, value: ParamValue, index: *mut int32) -> tresult {
        let mut p = self.points.borrow_mut();
        p.push((offset, value));
        if !index.is_null() {
            // SAFETY: not null; `index` is an optional caller-owned int32.
            unsafe { *index = p.len() as int32 - 1 };
        }
        kResultTrue
    }
}

/// The parameter changes for one block.
#[derive(Default)]
pub struct ParamChanges {
    queues: RefCell<Vec<ComWrapper<ParamQueue>>>,
}

impl ParamChanges {
    pub fn from_changes(changes: &[(u32, f64)]) -> Self {
        let pc = ParamChanges::default();
        for (id, v) in changes {
            let i = pc.find_or_add(*id);
            pc.queues.borrow()[i].points.borrow_mut().push((0, *v));
        }
        pc
    }

    fn find_or_add(&self, id: ParamID) -> usize {
        let mut q = self.queues.borrow_mut();
        if let Some(i) = q.iter().position(|x| x.id == id) {
            return i;
        }
        q.push(ComWrapper::new(ParamQueue {
            id,
            points: RefCell::new(Vec::new()),
        }));
        q.len() - 1
    }

    /// Every change, as (id, last value).
    pub fn last_values(&self) -> Vec<(u32, f64)> {
        self.queues
            .borrow()
            .iter()
            .filter_map(|q| q.points.borrow().last().map(|p| (q.id, p.1)))
            .collect()
    }
}

impl Class for ParamChanges {
    type Interfaces = (IParameterChanges,);
}

impl IParameterChangesTrait for ParamChanges {
    unsafe fn getParameterCount(&self) -> int32 {
        self.queues.borrow().len() as int32
    }

    unsafe fn getParameterData(&self, index: int32) -> *mut IParamValueQueue {
        match self.queues.borrow().get(index.max(0) as usize) {
            Some(q) => q
                .as_com_ref::<IParamValueQueue>()
                .map_or(ptr::null_mut(), |r| r.as_ptr()),
            None => ptr::null_mut(),
        }
    }

    unsafe fn addParameterData(
        &self,
        id: *const ParamID,
        index: *mut int32,
    ) -> *mut IParamValueQueue {
        if id.is_null() {
            return ptr::null_mut();
        }
        // SAFETY: not null (checked above); addParameterData's id points to
        // one ParamID owned by the caller, read once here.
        let i = self.find_or_add(unsafe { *id });
        if !index.is_null() {
            // SAFETY: not null; `index` is an optional caller-owned int32.
            unsafe { *index = i as int32 };
        }
        // SAFETY: our own method; it touches no raw pointers.
        unsafe { self.getParameterData(i as int32) }
    }
}

/// The note events for one block.
#[derive(Default)]
pub struct EventList {
    pub events: RefCell<Vec<Event>>,
}

impl Class for EventList {
    type Interfaces = (IEventList,);
}

impl IEventListTrait for EventList {
    unsafe fn getEventCount(&self) -> int32 {
        self.events.borrow().len() as int32
    }

    unsafe fn getEvent(&self, index: int32, e: *mut Event) -> tresult {
        match self.events.borrow().get(index.max(0) as usize) {
            Some(ev) if !e.is_null() => {
                // SAFETY: not null (match guard); getEvent's `e` is a
                // caller-owned Event to copy into (Event is Copy).
                unsafe { *e = *ev };
                kResultOk
            }
            _ => kResultFalse,
        }
    }

    unsafe fn addEvent(&self, e: *mut Event) -> tresult {
        if e.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: not null (checked above); addEvent's `e` points to one
        // Event owned by the caller, copied out here (Event is Copy).
        self.events.borrow_mut().push(unsafe { *e });
        kResultOk
    }
}

/// What the plug-in's window reports, for the host to act on.
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    Param(u32, f64),
    Latency,
    Dirty,
    Resize(i32, i32),
}

/// Receives edits made in the plug-in's own window, and restart requests.
pub struct Handler {
    pub edits: Arc<Mutex<Vec<Edit>>>,
}

impl Class for Handler {
    type Interfaces = (IComponentHandler,);
}

impl IComponentHandlerTrait for Handler {
    unsafe fn beginEdit(&self, _id: ParamID) -> tresult {
        kResultOk
    }

    unsafe fn performEdit(&self, id: ParamID, value: ParamValue) -> tresult {
        if let Ok(mut e) = self.edits.lock() {
            e.push(Edit::Param(id, value));
        }
        kResultOk
    }

    unsafe fn endEdit(&self, _id: ParamID) -> tresult {
        kResultOk
    }

    unsafe fn restartComponent(&self, flags: int32) -> tresult {
        use vst3::Steinberg::Vst::RestartFlags_::{
            kLatencyChanged, kParamValuesChanged, kReloadComponent,
        };
        if let Ok(mut e) = self.edits.lock() {
            if flags & kLatencyChanged as int32 != 0 {
                e.push(Edit::Latency);
            }
            if flags & (kParamValuesChanged as int32 | kReloadComponent as int32) != 0 {
                e.push(Edit::Dirty);
            }
        }
        kResultOk
    }
}

/// The window frame a plug-in view asks to resize.
pub struct Frame {
    pub edits: Arc<Mutex<Vec<Edit>>>,
}

impl Class for Frame {
    type Interfaces = (IPlugFrame,);
}

impl IPlugFrameTrait for Frame {
    unsafe fn resizeView(&self, view: *mut IPlugView, size: *mut ViewRect) -> tresult {
        if size.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: not null (checked above); resizeView's size is the
        // plug-in's own ViewRect, read once (ViewRect is Copy).
        let r = unsafe { *size };
        if let Ok(mut e) = self.edits.lock() {
            e.push(Edit::Resize(r.right - r.left, r.bottom - r.top));
        }
        // SAFETY: `view` is the plug-in's own IPlugView, alive for this call
        // (it is the object calling us); from_raw returns None for null and
        // borrows without taking a reference count.
        if let Some(v) = unsafe { vst3::ComRef::from_raw(view) } {
            use vst3::Steinberg::IPlugViewTrait;
            let mut r = r;
            // SAFETY: a live view (above) and a valid ViewRect on our stack.
            unsafe { v.onSize(&mut r) };
        }
        kResultTrue
    }
}

#[derive(Default)]
pub struct Message {
    id: RefCell<Vec<u8>>,
    attrs: ComWrapperCell,
}

/// Lazily made attribute list, shared with whoever asks for it.
#[derive(Default)]
struct ComWrapperCell(RefCell<Option<ComWrapper<Attributes>>>);

impl Class for Message {
    type Interfaces = (IMessage,);
}

impl IMessageTrait for Message {
    unsafe fn getMessageID(&self) -> FIDString {
        let id = self.id.borrow();
        if id.is_empty() {
            return c"".as_ptr();
        }
        id.as_ptr() as FIDString
    }

    unsafe fn setMessageID(&self, id: FIDString) {
        let mut v = if id.is_null() {
            Vec::new()
        } else {
            // SAFETY: not null (checked above); setMessageID takes a
            // nul-terminated C string the caller keeps alive for this call.
            // It is copied, so nothing keeps the pointer.
            unsafe { CStr::from_ptr(id) }.to_bytes().to_vec()
        };
        v.push(0);
        *self.id.borrow_mut() = v;
    }

    unsafe fn getAttributes(&self) -> *mut IAttributeList {
        let mut slot = self.attrs.0.borrow_mut();
        let a = slot.get_or_insert_with(|| ComWrapper::new(Attributes::default()));
        a.as_com_ref::<IAttributeList>()
            .map_or(ptr::null_mut(), |r| r.as_ptr())
    }
}

enum Attr {
    Int(i64),
    Float(f64),
    Str(Vec<TChar>),
    Bin(Vec<u8>),
}

#[derive(Default)]
pub struct Attributes {
    map: RefCell<HashMap<Vec<u8>, Attr>>,
}

impl Class for Attributes {
    type Interfaces = (IAttributeList,);
}

/// An attribute id, copied.
///
/// # Safety
/// `id` must be null or a nul-terminated C string valid for the call, as
/// IAttributeList's AttrID arguments are.
unsafe fn key(id: *const std::ffi::c_char) -> Vec<u8> {
    if id.is_null() {
        Vec::new()
    } else {
        // SAFETY: not null, and nul-terminated per this function's contract.
        unsafe { CStr::from_ptr(id) }.to_bytes().to_vec()
    }
}

impl IAttributeListTrait for Attributes {
    unsafe fn setInt(&self, id: *const std::ffi::c_char, value: int64) -> tresult {
        // SAFETY: `id` is an AttrID from the caller (see `key`).
        self.map
            .borrow_mut()
            .insert(unsafe { key(id) }, Attr::Int(value));
        kResultOk
    }

    unsafe fn getInt(&self, id: *const std::ffi::c_char, value: *mut int64) -> tresult {
        if value.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: `id` is an AttrID from the caller (see `key`).
        match self.map.borrow().get(&unsafe { key(id) }) {
            Some(Attr::Int(v)) => {
                // SAFETY: not null (checked above); a caller-owned int64.
                unsafe { *value = *v };
                kResultOk
            }
            _ => kResultFalse,
        }
    }

    unsafe fn setFloat(&self, id: *const std::ffi::c_char, value: f64) -> tresult {
        // SAFETY: `id` is an AttrID from the caller (see `key`).
        self.map
            .borrow_mut()
            .insert(unsafe { key(id) }, Attr::Float(value));
        kResultOk
    }

    unsafe fn getFloat(&self, id: *const std::ffi::c_char, value: *mut f64) -> tresult {
        if value.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: `id` is an AttrID from the caller (see `key`).
        match self.map.borrow().get(&unsafe { key(id) }) {
            Some(Attr::Float(v)) => {
                // SAFETY: not null (checked above); a caller-owned f64.
                unsafe { *value = *v };
                kResultOk
            }
            _ => kResultFalse,
        }
    }

    unsafe fn setString(&self, id: *const std::ffi::c_char, s: *const TChar) -> tresult {
        let mut v = Vec::new();
        if !s.is_null() {
            let mut i = 0;
            // SAFETY: not null (checked above); setString takes a
            // nul-terminated UTF-16 string, so every unit up to and
            // including the terminator is readable and nothing past it is
            // touched.
            while unsafe { *s.add(i) } != 0 {
                // SAFETY: as above; index i is before the terminator.
                v.push(unsafe { *s.add(i) });
                i += 1;
            }
        }
        v.push(0);
        // SAFETY: `id` is an AttrID from the caller (see `key`).
        self.map
            .borrow_mut()
            .insert(unsafe { key(id) }, Attr::Str(v));
        kResultOk
    }

    unsafe fn getString(
        &self,
        id: *const std::ffi::c_char,
        s: *mut TChar,
        size: uint32,
    ) -> tresult {
        if s.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: `id` is an AttrID from the caller (see `key`).
        match self.map.borrow().get(&unsafe { key(id) }) {
            Some(Attr::Str(v)) => {
                let room = size as usize / std::mem::size_of::<TChar>();
                if room == 0 {
                    return kResultFalse;
                }
                let n = v.len().min(room);
                // SAFETY: getString's `s` (not null, checked above) is a
                // caller buffer of `size` bytes, so `room` TChars; n <= room
                // and n <= v.len(), so both ranges are in bounds, and the
                // plug-in's buffer cannot be our Vec. n >= 1 (room >= 1 and
                // v always holds at least its terminator), so n - 1 is the
                // last unit written, inside the buffer.
                unsafe {
                    ptr::copy_nonoverlapping(v.as_ptr(), s, n);
                    *s.add(n - 1) = 0;
                }
                kResultOk
            }
            _ => kResultFalse,
        }
    }

    unsafe fn setBinary(
        &self,
        id: *const std::ffi::c_char,
        data: *const c_void,
        size: uint32,
    ) -> tresult {
        let v = if data.is_null() {
            Vec::new()
        } else {
            // SAFETY: not null (checked above); setBinary passes `size`
            // readable bytes at `data`, copied here so nothing keeps the
            // pointer. A u8 slice has no alignment requirement.
            unsafe { std::slice::from_raw_parts(data as *const u8, size as usize) }.to_vec()
        };
        // SAFETY: `id` is an AttrID from the caller (see `key`).
        self.map
            .borrow_mut()
            .insert(unsafe { key(id) }, Attr::Bin(v));
        kResultOk
    }

    unsafe fn getBinary(
        &self,
        id: *const std::ffi::c_char,
        data: *mut *const c_void,
        size: *mut uint32,
    ) -> tresult {
        if data.is_null() || size.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: `id` is an AttrID from the caller (see `key`).
        match self.map.borrow().get(&unsafe { key(id) }) {
            Some(Attr::Bin(v)) => {
                // The pointer handed out points into the list, which lives as
                // long as the message. VST3 says it stays valid only until
                // the attribute is set again or the list is released, which
                // matches: a later setBinary replaces (and frees) this Vec.
                // SAFETY: both out-pointers not null (checked above) and
                // caller-owned, valid for one write each.
                unsafe {
                    *data = v.as_ptr() as *const c_void;
                    *size = v.len() as uint32;
                }
                kResultOk
            }
            _ => kResultFalse,
        }
    }
}
