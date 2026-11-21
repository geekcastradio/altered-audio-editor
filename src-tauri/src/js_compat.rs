// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (c) The GeekCast Radio Network, LLC.
// This file is part of Altered Audio Editor.

//! Exact copies of the few JavaScript number behaviors the Rust ports of the
//! interface rules must match, so the shared fixtures agree in both languages.

/// JavaScript's `Math.round`: halves go toward positive infinity.
pub fn js_round(x: f64) -> f64 {
    let f = x.floor();
    if x - f >= 0.5 {
        f + 1.0
    } else {
        f
    }
}

/// JavaScript's `Number.prototype.toFixed(1)`: the nearest tenth of the exact
/// value, and an exact tie goes up in size (away from zero for negatives).
pub fn js_to_fixed1(x: f64) -> String {
    if x == 0.0 {
        // Negative zero has no sign in JavaScript's output.
        return "0.0".to_string();
    }
    if x < 0.0 {
        return format!("-{}", js_to_fixed1(-x));
    }
    let exact = format!("{x:.60}");
    let (whole, frac) = exact.split_once('.').unwrap_or((&exact, ""));
    let mut digits = frac.bytes();
    let tenth = digits.next().map_or(0, |d| u128::from(d - b'0'));
    let rest: Vec<u8> = digits.collect();
    let tie = rest.first() == Some(&b'5') && rest[1..].iter().all(|&d| d == b'0');
    if !tie {
        return format!("{x:.1}");
    }
    let n = whole.parse::<u128>().unwrap_or(0) * 10 + tenth + 1;
    format!("{}.{}", n / 10, n % 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_round_sends_halves_up() {
        assert_eq!(js_round(2.5), 3.0);
        assert_eq!(js_round(-2.5), -2.0);
        assert_eq!(js_round(-7.5), -7.0);
        assert_eq!(js_round(0.4), 0.0);
        assert_eq!(js_round(-0.6), -1.0);
    }

    #[test]
    fn js_to_fixed1_matches_javascript() {
        assert_eq!(js_to_fixed1(0.25), "0.3");
        assert_eq!(js_to_fixed1(2.25), "2.3");
        assert_eq!(js_to_fixed1(-2.25), "-2.3");
        assert_eq!(js_to_fixed1(59.25), "59.3");
        assert_eq!(js_to_fixed1(3.0), "3.0");
        assert_eq!(js_to_fixed1(5.5), "5.5");
        assert_eq!(js_to_fixed1(0.05), "0.1");
        assert_eq!(js_to_fixed1(0.04), "0.0");
        assert_eq!(js_to_fixed1(-0.04), "-0.0");
        assert_eq!(js_to_fixed1(9.95), "9.9");
        assert_eq!(js_to_fixed1(0.0), "0.0");
        assert_eq!(js_to_fixed1(-0.0), "0.0");
    }
}
