// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Hardware controllers: MIDI keyboards, fader boxes and control surfaces
//! (a Behringer BCF2000, a Mackie Control and the like) used as remote
//! controls. Messages come in, are decoded into three kinds (control
//! change, note, pitch bend) and wait in a short queue that the interface
//! empties many times a second. Nothing here plays or records sound, and
//! nothing leaves the computer. When feedback is switched on, short MIDI
//! messages also go back to the device (fader positions for motorized
//! faders, button and ring lights); nothing else is ever sent.

use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use midir::{Ignore, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use serde::Serialize;

/// What a message was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A knob, slider or relative encoder. `number` is the controller, `value` 0 to 127.
    Cc,
    /// A key or button. `number` is the note, `value` 0 for released, 1 to 127 for pressed.
    Note,
    /// A motorized or plain fader in a Mackie Control: one per channel. `value` 0 to 16383.
    Bend,
}

/// One decoded message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub port: String,
    pub kind: Kind,
    /// MIDI channel, 0 to 15.
    pub channel: u8,
    pub number: u8,
    pub value: u16,
}

/// Decodes raw MIDI bytes. Anything that is not a control change, note or
/// pitch bend (clock, program changes, system messages) is dropped.
pub fn parse(port: &str, bytes: &[u8]) -> Option<Message> {
    let status = *bytes.first()?;
    let channel = status & 0x0f;
    let at = |i: usize| bytes.get(i).map(|b| b & 0x7f);
    let (kind, number, value) = match status & 0xf0 {
        0xb0 => (Kind::Cc, at(1)?, u16::from(at(2)?)),
        0x90 => (Kind::Note, at(1)?, u16::from(at(2)?)),
        // A note off is a note released.
        0x80 => (Kind::Note, at(1)?, 0),
        0xe0 => (Kind::Bend, 0, u16::from(at(1)?) | (u16::from(at(2)?) << 7)),
        _ => return None,
    };
    Some(Message {
        port: port.to_string(),
        kind,
        channel,
        number,
        value,
    })
}

/// The most messages kept before the oldest go: a fader dragged for a
/// long while must not grow the queue if the interface is busy.
const QUEUE_MAX: usize = 512;

/// Messages waiting for the interface.
#[derive(Default)]
pub struct Queue {
    items: VecDeque<Message>,
}

impl Queue {
    /// Adds a message. A fader (pitch bend) that moved again before the
    /// interface looked replaces its older value, since only where it
    /// ended up matters. Knobs are kept in order: a relative encoder
    /// sends how far it turned, and every step counts.
    pub fn push(&mut self, m: Message) {
        if m.kind == Kind::Bend {
            if let Some(old) = self
                .items
                .iter_mut()
                .find(|o| o.kind == Kind::Bend && o.channel == m.channel && o.port == m.port)
            {
                old.value = m.value;
                return;
            }
        }
        if self.items.len() >= QUEUE_MAX {
            self.items.pop_front();
        }
        self.items.push_back(m);
    }

    pub fn drain(&mut self) -> Vec<Message> {
        self.items.drain(..).collect()
    }
}

/// A name with its direction and trailing port number set aside, for pairing
/// a device's input with its output: "MIDIIN2 (BCF2000)" and
/// "MIDIOUT2 (BCF2000)", "BCF2000 Port 1" and "BCF2000 Port 1".
fn same_device_name(s: &str) -> String {
    s.to_lowercase()
        .replace("midiin", "midiout")
        .trim()
        .to_string()
}

fn without_index(s: &str) -> String {
    match s.rsplit_once(' ') {
        Some((head, n)) if !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) => {
            head.trim_end().to_string()
        }
        _ => s.to_string(),
    }
}

/// Which of the output `ports` goes with the input called `wanted`: the same
/// name first, then the same name apart from the port number. Never a guess
/// beyond that, so nothing is sent to a device that was not asked for.
pub fn match_port(wanted: &str, ports: &[String]) -> Option<usize> {
    let w = same_device_name(wanted);
    if let Some(i) = ports.iter().position(|p| same_device_name(p) == w) {
        return Some(i);
    }
    let w = without_index(&w);
    ports
        .iter()
        .position(|p| without_index(&same_device_name(p)) == w)
}

/// Only plain three-byte channel messages go out: note off or on, aftertouch,
/// control change or pitch bend. No system exclusive, no program changes.
pub fn valid_out(bytes: &[u8]) -> bool {
    bytes.len() == 3
        && matches!(bytes[0] & 0xf0, 0x80 | 0x90 | 0xa0 | 0xb0 | 0xe0)
        && bytes[0] >= 0x80
        && bytes[1] < 0x80
        && bytes[2] < 0x80
}

/// The open input ports and what they received.
#[derive(Default)]
pub struct Controllers {
    open: Mutex<Vec<(String, MidiInputConnection<()>)>>,
    queue: Arc<Mutex<Queue>>,
    /// Output ports opened for feedback, by the input name they were asked for.
    outs: Mutex<Vec<(String, MidiOutputConnection)>>,
    /// Inputs that have no matching output, so they are not looked up again until the list changes.
    no_output: Mutex<HashSet<String>>,
}

impl Controllers {
    /// The names of the MIDI inputs connected now.
    pub fn ports() -> Vec<String> {
        let Ok(input) = MidiInput::new("Altered Audio Editor") else {
            return Vec::new();
        };
        input
            .ports()
            .iter()
            .filter_map(|p| input.port_name(p).ok())
            .collect()
    }

    /// Listens to exactly the ports in `names` (the rest are closed).
    /// Returns the ones that opened.
    pub fn open(&self, names: &[String]) -> Vec<String> {
        let Ok(mut open) = self.open.lock() else {
            return Vec::new();
        };
        if let Ok(mut outs) = self.outs.lock() {
            outs.retain(|(n, _)| names.contains(n));
        }
        if let Ok(mut none) = self.no_output.lock() {
            none.clear();
        }
        // Close the ones no longer wanted.
        let (keep, drop): (Vec<_>, Vec<_>) = open.drain(..).partition(|(n, _)| names.contains(n));
        for (_, c) in drop {
            c.close();
        }
        *open = keep;
        for name in names {
            if open.iter().any(|(n, _)| n == name) {
                continue;
            }
            let Ok(mut input) = MidiInput::new("Altered Audio Editor") else {
                continue;
            };
            // Clock and sensing are noise to us.
            input.ignore(Ignore::TimeAndActiveSense);
            let Some(port) = input
                .ports()
                .into_iter()
                .find(|p| input.port_name(p).is_ok_and(|n| n == *name))
            else {
                continue;
            };
            let queue = Arc::clone(&self.queue);
            let label = name.clone();
            let conn = input.connect(
                &port,
                "controller",
                move |_, bytes, _| {
                    if let Some(m) = parse(&label, bytes) {
                        if let Ok(mut q) = queue.lock() {
                            q.push(m);
                        }
                    }
                },
                (),
            );
            if let Ok(c) = conn {
                open.push((name.clone(), c));
            }
        }
        open.iter().map(|(n, _)| n.clone()).collect()
    }

    /// Sends messages back to the device whose input is `port`, opening its
    /// output on first use. Returns false when it has no output or a send failed.
    pub fn send(&self, port: &str, messages: &[Vec<u8>]) -> bool {
        let Ok(mut outs) = self.outs.lock() else {
            return false;
        };
        if !outs.iter().any(|(n, _)| n == port) {
            if self.no_output.lock().is_ok_and(|n| n.contains(port)) {
                return false;
            }
            let opened = MidiOutput::new("Altered Audio Editor")
                .ok()
                .and_then(|out| {
                    let ports = out.ports();
                    let names: Vec<String> = ports
                        .iter()
                        .map(|p| out.port_name(p).unwrap_or_default())
                        .collect();
                    let at = match_port(port, &names)?;
                    out.connect(ports.get(at)?, "controller-out").ok()
                });
            match opened {
                Some(conn) => outs.push((port.to_string(), conn)),
                None => {
                    if let Ok(mut none) = self.no_output.lock() {
                        none.insert(port.to_string());
                    }
                    return false;
                }
            }
        }
        let Some(i) = outs.iter().position(|(n, _)| n == port) else {
            return false;
        };
        for m in messages.iter().filter(|m| valid_out(m)) {
            if outs[i].1.send(m).is_err() {
                // The device went away: forget the connection so the next send looks again.
                outs.remove(i);
                return false;
            }
        }
        true
    }

    /// Takes what arrived since the last call, oldest first.
    pub fn events(&self) -> Vec<Message> {
        self.queue.lock().map(|mut q| q.drain()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(kind: Kind, channel: u8, number: u8, value: u16) -> Message {
        Message {
            port: "p".into(),
            kind,
            channel,
            number,
            value,
        }
    }

    #[test]
    fn decodes_control_changes_notes_and_pitch_bend() {
        assert_eq!(parse("p", &[0xb2, 7, 100]), Some(msg(Kind::Cc, 2, 7, 100)));
        assert_eq!(
            parse("p", &[0x90, 60, 90]),
            Some(msg(Kind::Note, 0, 60, 90))
        );
        // A note on with no velocity and a note off both mean released.
        assert_eq!(parse("p", &[0x90, 60, 0]), Some(msg(Kind::Note, 0, 60, 0)));
        assert_eq!(parse("p", &[0x83, 60, 64]), Some(msg(Kind::Note, 3, 60, 0)));
        // Pitch bend is two 7-bit halves, low first. Center is 8192.
        assert_eq!(
            parse("p", &[0xe1, 0, 64]),
            Some(msg(Kind::Bend, 1, 0, 8192))
        );
        assert_eq!(
            parse("p", &[0xe0, 127, 127]),
            Some(msg(Kind::Bend, 0, 0, 16383))
        );
    }

    #[test]
    fn an_input_is_paired_with_its_own_output() {
        let outs = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        // Same name.
        assert_eq!(match_port("BCF2000", &outs(&["Other", "BCF2000"])), Some(1));
        // Windows names the two directions differently.
        assert_eq!(
            match_port("MIDIIN2 (BCF2000)", &outs(&["MIDIOUT2 (BCF2000)"])),
            Some(0)
        );
        // Only the port number differs.
        assert_eq!(match_port("BCF2000 1", &outs(&["BCF2000 0"])), Some(0));
        // An exact match wins over a looser one.
        assert_eq!(
            match_port("BCF2000 1", &outs(&["BCF2000 0", "BCF2000 1"])),
            Some(1)
        );
        // Nothing similar: nothing is chosen.
        assert_eq!(match_port("BCF2000", &outs(&["Keyboard", "Synth"])), None);
    }

    #[test]
    fn only_plain_channel_messages_go_out() {
        assert!(valid_out(&[0xe0, 0, 64]));
        assert!(valid_out(&[0x90, 16, 127]));
        assert!(valid_out(&[0xb0, 48, 0x11]));
        assert!(!valid_out(&[0xf0, 0, 0x7f]));
        assert!(!valid_out(&[0xc0, 5, 0]));
        assert!(!valid_out(&[0x90, 200, 1]));
        assert!(!valid_out(&[0x90, 1]));
        assert!(!valid_out(&[0x90, 1, 2, 3]));
        assert!(!valid_out(&[0x10, 1, 2]));
    }

    #[test]
    fn drops_what_it_does_not_use() {
        assert_eq!(parse("p", &[]), None);
        assert_eq!(parse("p", &[0xc0, 5]), None);
        assert_eq!(parse("p", &[0xf8]), None);
        assert_eq!(parse("p", &[0xb0, 7]), None);
    }

    #[test]
    fn a_fader_keeps_only_where_it_ended_but_knobs_keep_every_step() {
        let mut q = Queue::default();
        q.push(msg(Kind::Bend, 0, 0, 100));
        q.push(msg(Kind::Bend, 1, 0, 5));
        q.push(msg(Kind::Bend, 0, 0, 300));
        q.push(msg(Kind::Cc, 0, 16, 1));
        q.push(msg(Kind::Cc, 0, 16, 1));
        let got = q.drain();
        assert_eq!(got.len(), 4);
        assert_eq!(got[0].value, 300);
        assert!(q.drain().is_empty());
    }

    #[test]
    fn the_queue_never_grows_without_bound() {
        let mut q = Queue::default();
        for i in 0..2000u16 {
            q.push(msg(Kind::Cc, 0, 1, i % 128));
        }
        assert_eq!(q.drain().len(), QUEUE_MAX);
    }
}
