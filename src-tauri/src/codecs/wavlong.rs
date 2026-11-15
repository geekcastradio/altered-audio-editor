// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Headers for WAV files past 4 GB: RF64 (the EBU extension of WAV) and Sony Wave64. The writer
//! builds a header for a known data size and picks the plain WAV header whenever the data fits,
//! so ordinary files stay ordinary. The reader finds the format and where the audio starts in
//! plain WAV, RF64 and Wave64.

const U32_MAX: u64 = u32::MAX as u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    Wav,
    Rf64,
    W64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    pub channels: u16,
    pub rate: u32,
    pub bits: u16,
    pub float: bool,
    /// Speaker mask for more than two channels; zero means "not stated".
    pub mask: u32,
}

impl Layout {
    pub fn frame_bytes(&self) -> u64 {
        u64::from(self.channels) * u64::from(self.bits / 8)
    }

    fn extensible(&self) -> bool {
        self.channels > 2 || self.bits > 16
    }
}

/// The container a file of this size needs: plain WAV below 4 GB, RF64 above, or Wave64 when asked.
pub fn container_for(data_bytes: u64, want_w64: bool) -> Container {
    if want_w64 {
        Container::W64
    } else if data_bytes + 4096 > U32_MAX {
        Container::Rf64
    } else {
        Container::Wav
    }
}

const GUID_RIFF: [u8; 16] = [
    0x72, 0x69, 0x66, 0x66, 0x2E, 0x91, 0xCF, 0x11, 0xA5, 0xD6, 0x28, 0xDB, 0x04, 0xC1, 0x00, 0x00,
];
const GUID_WAVE: [u8; 16] = [
    0x77, 0x61, 0x76, 0x65, 0xF3, 0xAC, 0xD3, 0x11, 0x8C, 0xD1, 0x00, 0xC0, 0x4F, 0x8E, 0xDB, 0x8A,
];
const GUID_FMT: [u8; 16] = [
    0x66, 0x6D, 0x74, 0x20, 0xF3, 0xAC, 0xD3, 0x11, 0x8C, 0xD1, 0x00, 0xC0, 0x4F, 0x8E, 0xDB, 0x8A,
];
const GUID_DATA: [u8; 16] = [
    0x64, 0x61, 0x74, 0x61, 0xF3, 0xAC, 0xD3, 0x11, 0x8C, 0xD1, 0x00, 0xC0, 0x4F, 0x8E, 0xDB, 0x8A,
];
/// The rest of the KSDATAFORMAT_SUBTYPE GUID after the two-byte format code.
const SUBTYPE_TAIL: [u8; 14] = [
    0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71,
];

fn fmt_body(l: &Layout) -> Vec<u8> {
    let ext = l.extensible();
    let block = (l.frame_bytes()) as u16;
    let mut b = Vec::with_capacity(40);
    let tag: u16 = if ext {
        0xFFFE
    } else if l.float {
        3
    } else {
        1
    };
    b.extend_from_slice(&tag.to_le_bytes());
    b.extend_from_slice(&l.channels.to_le_bytes());
    b.extend_from_slice(&l.rate.to_le_bytes());
    b.extend_from_slice(&(l.rate * u32::from(block)).to_le_bytes());
    b.extend_from_slice(&block.to_le_bytes());
    b.extend_from_slice(&l.bits.to_le_bytes());
    if ext {
        b.extend_from_slice(&22u16.to_le_bytes());
        b.extend_from_slice(&l.bits.to_le_bytes()); // valid bits
        b.extend_from_slice(&l.mask.to_le_bytes());
        b.extend_from_slice(&(if l.float { 3u16 } else { 1u16 }).to_le_bytes());
        b.extend_from_slice(&SUBTYPE_TAIL);
    }
    b
}

/// The header to write before `data_bytes` of audio. The caller writes the audio after it and, for
/// plain WAV and Wave64, pads an odd data size to the chunk boundary itself (`pad_after`).
pub fn header(layout: &Layout, data_bytes: u64, container: Container) -> Vec<u8> {
    let fmt = fmt_body(layout);
    let mut out = Vec::new();
    match container {
        Container::Wav => {
            let riff = 4 + 8 + fmt.len() as u64 + 8 + data_bytes + (data_bytes & 1);
            out.extend_from_slice(b"RIFF");
            out.extend_from_slice(&(riff.min(U32_MAX) as u32).to_le_bytes());
            out.extend_from_slice(b"WAVE");
            out.extend_from_slice(b"fmt ");
            out.extend_from_slice(&(fmt.len() as u32).to_le_bytes());
            out.extend_from_slice(&fmt);
            out.extend_from_slice(b"data");
            out.extend_from_slice(&(data_bytes.min(U32_MAX) as u32).to_le_bytes());
        }
        Container::Rf64 => {
            // ds64 holds the real 64-bit sizes; the 32-bit fields say "look in ds64".
            let ds64_len = 28u32;
            let riff = 4
                + (8 + u64::from(ds64_len))
                + 8
                + fmt.len() as u64
                + 8
                + data_bytes
                + (data_bytes & 1);
            out.extend_from_slice(b"RF64");
            out.extend_from_slice(&u32::MAX.to_le_bytes());
            out.extend_from_slice(b"WAVE");
            out.extend_from_slice(b"ds64");
            out.extend_from_slice(&ds64_len.to_le_bytes());
            out.extend_from_slice(&riff.to_le_bytes());
            out.extend_from_slice(&data_bytes.to_le_bytes());
            out.extend_from_slice(&(data_bytes / layout.frame_bytes().max(1)).to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes());
            out.extend_from_slice(b"fmt ");
            out.extend_from_slice(&(fmt.len() as u32).to_le_bytes());
            out.extend_from_slice(&fmt);
            out.extend_from_slice(b"data");
            out.extend_from_slice(&u32::MAX.to_le_bytes());
        }
        Container::W64 => {
            // Chunk sizes count their own 24-byte header and every chunk is padded to 8 bytes.
            let pad8 = |n: u64| (8 - n % 8) % 8;
            let fmt_chunk = 24 + fmt.len() as u64;
            let data_chunk = 24 + data_bytes;
            let total = 16 + 8 + 16 + fmt_chunk + pad8(fmt_chunk) + data_chunk + pad8(data_chunk);
            out.extend_from_slice(&GUID_RIFF);
            out.extend_from_slice(&total.to_le_bytes());
            out.extend_from_slice(&GUID_WAVE);
            out.extend_from_slice(&GUID_FMT);
            out.extend_from_slice(&fmt_chunk.to_le_bytes());
            out.extend_from_slice(&fmt);
            out.extend(std::iter::repeat_n(0u8, pad8(fmt_chunk) as usize));
            out.extend_from_slice(&GUID_DATA);
            out.extend_from_slice(&data_chunk.to_le_bytes());
        }
    }
    out
}

/// Bytes of padding to write after `data_bytes` of audio so the next chunk lines up.
pub fn pad_after(data_bytes: u64, container: Container) -> usize {
    match container {
        Container::Wav | Container::Rf64 => (data_bytes & 1) as usize,
        Container::W64 => ((8 - (24 + data_bytes) % 8) % 8) as usize,
    }
}

/// Bytes of the JUNK chunk reserved after `WAVE`. It becomes `ds64` if the file grows past 4 GB.
const RESERVED: usize = 28;

/// The header for a file whose length is not known yet: RIFF, a reserved 28 byte JUNK chunk,
/// `fmt ` and `data`, all sizes zero. Write the audio after it and call `close_header` at the end.
/// Players read the JUNK chunk as padding, so a file that stays small is an ordinary WAV.
pub fn open_header(layout: &Layout) -> Vec<u8> {
    let fmt = fmt_body(layout);
    let mut out = Vec::with_capacity(12 + 8 + RESERVED + 8 + fmt.len() + 8);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"JUNK");
    out.extend_from_slice(&(RESERVED as u32).to_le_bytes());
    out.extend(std::iter::repeat_n(0u8, RESERVED));
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&(fmt.len() as u32).to_le_bytes());
    out.extend_from_slice(&fmt);
    out.extend_from_slice(b"data");
    out.extend_from_slice(&0u32.to_le_bytes());
    out
}

/// Fills in the sizes of a file started with `open_header`, once `data_bytes` of audio are written
/// (the caller has also written the pad byte for an odd size). Past 4 GB the file becomes RF64.
pub fn close_header<F: std::io::Write + std::io::Seek>(
    f: &mut F,
    layout: &Layout,
    data_bytes: u64,
) -> std::io::Result<Container> {
    use std::io::SeekFrom;
    let fmt_len = fmt_body(layout).len() as u64;
    let data_at = 12 + 8 + RESERVED as u64 + 8 + fmt_len + 4;
    let riff = 4 + 8 + RESERVED as u64 + 8 + fmt_len + 8 + data_bytes + (data_bytes & 1);
    if riff <= U32_MAX {
        f.seek(SeekFrom::Start(4))?;
        f.write_all(&(riff as u32).to_le_bytes())?;
        f.seek(SeekFrom::Start(data_at))?;
        f.write_all(&(data_bytes as u32).to_le_bytes())?;
        f.seek(SeekFrom::End(0))?;
        return Ok(Container::Wav);
    }
    f.seek(SeekFrom::Start(0))?;
    f.write_all(b"RF64")?;
    f.write_all(&u32::MAX.to_le_bytes())?;
    f.seek(SeekFrom::Start(12))?;
    f.write_all(b"ds64")?;
    f.write_all(&(RESERVED as u32).to_le_bytes())?;
    f.write_all(&riff.to_le_bytes())?;
    f.write_all(&data_bytes.to_le_bytes())?;
    f.write_all(&(data_bytes / layout.frame_bytes().max(1)).to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    f.seek(SeekFrom::Start(data_at))?;
    f.write_all(&u32::MAX.to_le_bytes())?;
    f.seek(SeekFrom::End(0))?;
    Ok(Container::Rf64)
}

/// The speaker mask used when none is given: one bit per channel, as common WAV writers do.
pub fn default_mask(channels: u16) -> u32 {
    (0..u32::from(channels.min(18))).fold(0, |a, c| a | (1 << c))
}

/// A WAV writer for audio of any length: an ordinary WAV while the audio fits in 4 GB, RF64 when
/// it does not. Samples go in one at a time; `finish` writes the sizes.
pub struct WavOut {
    out: std::io::BufWriter<std::fs::File>,
    layout: Layout,
    bytes: u64,
}

impl WavOut {
    pub fn create(
        path: &std::path::Path,
        channels: u16,
        rate: u32,
        bits: u16,
        float: bool,
    ) -> std::io::Result<Self> {
        use std::io::Write;
        let layout = Layout {
            channels,
            rate,
            bits,
            float,
            mask: default_mask(channels),
        };
        let mut out = std::io::BufWriter::new(std::fs::File::create(path)?);
        out.write_all(&open_header(&layout))?;
        Ok(Self {
            out,
            layout,
            bytes: 0,
        })
    }

    fn put(&mut self, b: &[u8]) -> std::io::Result<()> {
        use std::io::Write;
        self.bytes += b.len() as u64;
        self.out.write_all(b)
    }

    /// An 8-bit sample. WAV stores 8-bit audio unsigned.
    pub fn put_i8(&mut self, v: i8) -> std::io::Result<()> {
        self.put(&[(v as u8) ^ 0x80])
    }

    pub fn put_i16(&mut self, v: i16) -> std::io::Result<()> {
        self.put(&v.to_le_bytes())
    }

    /// A 24-bit sample held in an i32.
    pub fn put_i24(&mut self, v: i32) -> std::io::Result<()> {
        self.put(&v.to_le_bytes()[..3])
    }

    pub fn put_f32(&mut self, v: f32) -> std::io::Result<()> {
        self.put(&v.to_le_bytes())
    }

    /// Writes the sizes and closes the file.
    pub fn finish(mut self) -> std::io::Result<Container> {
        use std::io::Write;
        if self.bytes & 1 == 1 {
            self.out.write_all(&[0])?;
        }
        self.out.flush()?;
        let mut f = self.out.into_inner().map_err(|e| e.into_error())?;
        let c = close_header(&mut f, &self.layout, self.bytes)?;
        f.flush()?;
        Ok(c)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    pub container: Container,
    pub layout: Layout,
    /// Where the audio starts in the file.
    pub data_offset: u64,
    pub data_bytes: u64,
}

impl Info {
    pub fn frames(&self) -> u64 {
        self.data_bytes / self.layout.frame_bytes().max(1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadError {
    NotWav,
    Truncated,
    Unsupported(&'static str),
}

fn le16(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}
fn le32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}
fn le64(b: &[u8], at: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(at..at + 8)?.try_into().ok()?))
}

fn read_fmt(body: &[u8]) -> Result<Layout, ReadError> {
    let tag = le16(body, 0).ok_or(ReadError::Truncated)?;
    let channels = le16(body, 2).ok_or(ReadError::Truncated)?;
    let rate = le32(body, 4).ok_or(ReadError::Truncated)?;
    let bits = le16(body, 14).ok_or(ReadError::Truncated)?;
    let (float, mask) = match tag {
        1 => (false, 0),
        3 => (true, 0),
        0xFFFE => {
            let mask = le32(body, 20).ok_or(ReadError::Truncated)?;
            let sub = le16(body, 24).ok_or(ReadError::Truncated)?;
            match sub {
                1 => (false, mask),
                3 => (true, mask),
                _ => return Err(ReadError::Unsupported("only PCM and float audio")),
            }
        }
        _ => return Err(ReadError::Unsupported("only PCM and float audio")),
    };
    if channels == 0 || rate == 0 || bits == 0 || bits % 8 != 0 {
        return Err(ReadError::Unsupported("an odd sample format"));
    }
    Ok(Layout {
        channels,
        rate,
        bits,
        float,
        mask,
    })
}

/// Reads the head of a file (a few KB is enough) and finds the format and the audio.
pub fn parse(head: &[u8], file_len: u64) -> Result<Info, ReadError> {
    if head.len() >= 16 && head[..16] == GUID_RIFF {
        return parse_w64(head, file_len);
    }
    let kind = head.get(..4).ok_or(ReadError::NotWav)?;
    let container = match kind {
        b"RIFF" => Container::Wav,
        // BW64 (the ITU broadcast file) has the same layout as RF64.
        b"RF64" | b"BW64" => Container::Rf64,
        _ => return Err(ReadError::NotWav),
    };
    if head.get(8..12) != Some(b"WAVE".as_slice()) {
        return Err(ReadError::NotWav);
    }
    let mut at = 12usize;
    let mut layout: Option<Layout> = None;
    let mut ds64_data: Option<u64> = None;
    loop {
        let id = head.get(at..at + 4).ok_or(ReadError::Truncated)?;
        let size = u64::from(le32(head, at + 4).ok_or(ReadError::Truncated)?);
        let body = at + 8;
        match id {
            b"ds64" => ds64_data = le64(head, body + 8),
            b"fmt " => {
                let end = body + size as usize;
                layout = Some(read_fmt(head.get(body..end).ok_or(ReadError::Truncated)?)?);
            }
            b"data" => {
                let layout = layout.ok_or(ReadError::Unsupported("audio before its format"))?;
                let mut bytes = size;
                if container == Container::Rf64 && size == U32_MAX {
                    bytes = ds64_data.ok_or(ReadError::Unsupported("RF64 without ds64"))?;
                }
                // A streamed or cut-off file says more than it holds.
                let available = file_len.saturating_sub(body as u64);
                return Ok(Info {
                    container,
                    layout,
                    data_offset: body as u64,
                    data_bytes: bytes.min(available),
                });
            }
            _ => {}
        }
        at = body + size as usize + (size as usize & 1);
    }
}

fn parse_w64(head: &[u8], file_len: u64) -> Result<Info, ReadError> {
    if head.get(24..40) != Some(GUID_WAVE.as_slice()) {
        return Err(ReadError::NotWav);
    }
    let mut at = 40usize;
    let mut layout: Option<Layout> = None;
    loop {
        let guid = head.get(at..at + 16).ok_or(ReadError::Truncated)?;
        let size = le64(head, at + 16).ok_or(ReadError::Truncated)?;
        let body = at + 24;
        if size < 24 {
            return Err(ReadError::Unsupported("a damaged chunk"));
        }
        let payload = (size - 24) as usize;
        if guid == GUID_FMT {
            layout = Some(read_fmt(
                head.get(body..body + payload).ok_or(ReadError::Truncated)?,
            )?);
        } else if guid == GUID_DATA {
            let layout = layout.ok_or(ReadError::Unsupported("audio before its format"))?;
            let available = file_len.saturating_sub(body as u64);
            return Ok(Info {
                container: Container::W64,
                layout,
                data_offset: body as u64,
                data_bytes: (size - 24).min(available),
            });
        }
        at = body + payload + (8 - (size as usize % 8)) % 8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stereo16() -> Layout {
        Layout {
            channels: 2,
            rate: 48_000,
            bits: 16,
            float: false,
            mask: 0,
        }
    }

    fn six24() -> Layout {
        Layout {
            channels: 6,
            rate: 48_000,
            bits: 24,
            float: false,
            mask: 0x3F,
        }
    }

    #[test]
    fn small_files_stay_plain_wav() {
        assert_eq!(container_for(1_000_000, false), Container::Wav);
        assert_eq!(container_for(5_000_000_000, false), Container::Rf64);
        assert_eq!(container_for(100, true), Container::W64);
        // Right at the edge the header itself would push a plain WAV past 4 GB.
        assert_eq!(container_for(U32_MAX - 100, false), Container::Rf64);
    }

    #[test]
    fn each_header_reads_back_the_same() {
        for (layout, bytes) in [(stereo16(), 400_000u64), (six24(), 3 * 6 * 48_000)] {
            for container in [Container::Wav, Container::Rf64, Container::W64] {
                let mut file = header(&layout, bytes, container);
                let offset = file.len() as u64;
                file.extend(std::iter::repeat_n(0u8, bytes as usize));
                file.extend(std::iter::repeat_n(0u8, pad_after(bytes, container)));
                let info = parse(&file[..file.len().min(4096)], file.len() as u64).unwrap();
                assert_eq!(info.container, container);
                assert_eq!(info.layout, layout, "{container:?}");
                assert_eq!(info.data_offset, offset, "{container:?}");
                assert_eq!(info.data_bytes, bytes, "{container:?}");
            }
        }
    }

    #[test]
    fn an_rf64_header_holds_sizes_past_4_gb() {
        let bytes = 6_000_000_000u64;
        let h = header(&stereo16(), bytes, Container::Rf64);
        assert_eq!(&h[..4], b"RF64");
        assert_eq!(&h[4..8], &[0xFF; 4]);
        // Pretend the file is that long: the reader trusts ds64.
        let info = parse(&h, bytes + h.len() as u64).unwrap();
        assert_eq!(info.data_bytes, bytes);
        assert_eq!(info.frames(), bytes / 4);
    }

    #[test]
    fn a_cut_off_file_reports_what_is_there() {
        let h = header(&stereo16(), 1_000_000, Container::Wav);
        let info = parse(&h, h.len() as u64 + 1_000).unwrap();
        assert_eq!(info.data_bytes, 1_000);
    }

    #[test]
    fn wave64_chunks_are_padded_to_eight() {
        let layout = Layout {
            channels: 1,
            rate: 44_100,
            bits: 16,
            float: false,
            mask: 0,
        };
        let h = header(&layout, 3, Container::W64);
        assert_eq!(
            h.len() % 8,
            0,
            "the data chunk starts on an 8-byte boundary"
        );
        assert_eq!(pad_after(3, Container::W64), 5);
        let total = u64::from_le_bytes(h[16..24].try_into().unwrap());
        assert_eq!(total % 8, 0);
    }

    #[test]
    fn float_and_extensible_formats_are_flagged() {
        let l = Layout {
            channels: 2,
            rate: 96_000,
            bits: 32,
            float: true,
            mask: 3,
        };
        let h = header(&l, 800, Container::Wav);
        let info = parse(&h, h.len() as u64 + 800).unwrap();
        assert!(info.layout.float);
        assert_eq!(info.layout.bits, 32);
    }

    #[test]
    fn other_files_are_refused() {
        assert_eq!(parse(b"hello world, not audio", 22), Err(ReadError::NotWav));
        assert_eq!(parse(b"RIFF\0\0\0\0AVI LIST", 16), Err(ReadError::NotWav));
        let mut h = header(&stereo16(), 100, Container::Wav);
        h.truncate(30);
        assert_eq!(parse(&h, 30), Err(ReadError::Truncated));
    }

    #[test]
    fn a_file_opened_and_closed_reads_back_small_and_large() {
        use std::io::Cursor;
        for (bytes, want) in [
            (400_000u64, Container::Wav),
            (6_000_000_000u64, Container::Rf64),
        ] {
            let layout = six24();
            let mut f = Cursor::new(open_header(&layout));
            f.set_position(f.get_ref().len() as u64);
            // Only the header is real; the audio length is claimed.
            let got = close_header(&mut f, &layout, bytes).unwrap();
            assert_eq!(got, want);
            let head = f.into_inner();
            let info = parse(&head, head.len() as u64 + bytes).unwrap();
            assert_eq!(info.container, want);
            assert_eq!(info.layout, layout);
            assert_eq!(info.data_bytes, bytes);
            assert_eq!(info.data_offset, head.len() as u64);
        }
    }

    #[test]
    fn a_small_closed_file_is_plain_wav_with_junk_before_fmt() {
        use std::io::Cursor;
        let layout = stereo16();
        let mut f = Cursor::new(open_header(&layout));
        f.set_position(f.get_ref().len() as u64);
        close_header(&mut f, &layout, 1_000).unwrap();
        let h = f.into_inner();
        assert_eq!(&h[..4], b"RIFF");
        assert_eq!(&h[12..16], b"JUNK");
        assert_eq!(&h[48..52], b"fmt ");
        assert_eq!(
            u32::from_le_bytes(h[4..8].try_into().unwrap()) as usize,
            h.len() - 8 + 1_000
        );
    }

    #[test]
    fn bw64_reads_like_rf64() {
        let mut h = header(&stereo16(), 6_000_000_000, Container::Rf64);
        h[..4].copy_from_slice(b"BW64");
        let info = parse(&h, 6_000_000_000 + h.len() as u64).unwrap();
        assert_eq!(info.container, Container::Rf64);
        assert_eq!(info.data_bytes, 6_000_000_000);
    }

    #[test]
    fn wav_out_reads_back_through_a_wav_reader() {
        let dir = std::env::temp_dir().join(format!("wavout-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let val = |k: i32| (k % 100) - 50;
        for (channels, bits, float) in [
            (1u16, 8u16, false),
            (2, 16, false),
            (2, 24, false),
            (6, 24, false),
            (2, 32, true),
        ] {
            let path = dir.join(format!("t-{channels}-{bits}.wav"));
            let mut w = WavOut::create(&path, channels, 44_100, bits, float).unwrap();
            let n = 50usize * usize::from(channels);
            for k in 0..n {
                let v = val(k as i32);
                match (bits, float) {
                    (8, _) => w.put_i8(v as i8).unwrap(),
                    (16, _) => w.put_i16((v * 300) as i16).unwrap(),
                    (24, _) => w.put_i24(v * 70_000).unwrap(),
                    _ => w.put_f32(v as f32 / 64.0).unwrap(),
                }
            }
            assert_eq!(w.finish().unwrap(), Container::Wav);
            let r = hound::WavReader::open(&path).unwrap();
            assert_eq!(r.spec().channels, channels);
            assert_eq!(r.spec().bits_per_sample, bits);
            assert_eq!(r.len() as usize, n);
            if float {
                let got: Vec<f32> = r.into_samples::<f32>().map(Result::unwrap).collect();
                assert_eq!(got[10], f32::from(val(10) as i16) / 64.0);
            } else {
                let got: Vec<i32> = r.into_samples::<i32>().map(Result::unwrap).collect();
                let v = val(10);
                let want = match bits {
                    8 => v,
                    16 => v * 300,
                    _ => v * 70_000,
                };
                assert_eq!(got[10], want, "{bits}-bit");
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
