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

/// JavaScript's `Number.prototype.toFixed(1)`. See `js_to_fixed`.
pub fn js_to_fixed1(x: f64) -> String {
    js_to_fixed(x, 1)
}

/// JavaScript's `Number.prototype.toFixed(digits)` for finite numbers: the
/// nearest value with that many decimals, and an exact tie goes up in size
/// (away from zero for negatives). Rust's own formatting breaks ties to even.
pub fn js_to_fixed(x: f64, digits: usize) -> String {
    if x == 0.0 {
        // Negative zero has no sign in JavaScript's output.
        return js_to_fixed_zero(digits);
    }
    if x < 0.0 {
        return format!("-{}", js_to_fixed(-x, digits));
    }
    let exact = format!("{x:.60}");
    let (whole, frac) = exact.split_once('.').unwrap_or((&exact, ""));
    let kept = &frac[..digits.min(frac.len())];
    let rest = &frac.as_bytes()[kept.len()..];
    let tie = rest.first() == Some(&b'5') && rest[1..].iter().all(|&d| d == b'0');
    if !tie {
        return format!("{x:.digits$}");
    }
    let scale = 10u128.pow(digits as u32);
    let n = whole.parse::<u128>().unwrap_or(0) * scale + kept.parse::<u128>().unwrap_or(0) + 1;
    if digits == 0 {
        n.to_string()
    } else {
        format!("{}.{:0width$}", n / scale, n % scale, width = digits)
    }
}

fn js_to_fixed_zero(digits: usize) -> String {
    if digits == 0 {
        "0".to_string()
    } else {
        format!("0.{}", "0".repeat(digits))
    }
}

/// JavaScript's string form of a number (`String(x)`, or `${x}`) for the
/// ranges the interface uses. Negative zero prints as "0". Numbers below 1e-6
/// or from 1e21 up, which JavaScript writes with an exponent, are not handled.
pub fn js_number_to_string(x: f64) -> String {
    if x == 0.0 {
        "0".to_string()
    } else if x.is_nan() {
        "NaN".to_string()
    } else if x.is_infinite() {
        if x > 0.0 { "Infinity" } else { "-Infinity" }.to_string()
    } else {
        format!("{x}")
    }
}

/// What JavaScript counts as white space (`\s`, and what `trim` removes).
pub fn is_js_space(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{0b}' | '\u{0c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}

/// JavaScript's `String.prototype.trim`.
pub fn js_trim(s: &str) -> &str {
    s.trim_matches(is_js_space)
}

/// JavaScript's `Number(text)` for plain text: a decimal number with an
/// optional sign and exponent, `0x`, `0o` and `0b` numbers, "Infinity", and
/// empty or blank text as 0. Anything else is None (NaN in JavaScript).
pub fn js_string_to_number(text: &str) -> Option<f64> {
    let t = js_trim(text);
    if t.is_empty() {
        return Some(0.0);
    }
    for (prefix, radix) in [
        ("0x", 16),
        ("0X", 16),
        ("0o", 8),
        ("0O", 8),
        ("0b", 2),
        ("0B", 2),
    ] {
        if let Some(rest) = t.strip_prefix(prefix) {
            return u64::from_str_radix(rest, radix).ok().map(|v| v as f64);
        }
    }
    let (sign, body) = match t.as_bytes()[0] {
        b'-' => (-1.0, &t[1..]),
        b'+' => (1.0, &t[1..]),
        _ => (1.0, t),
    };
    if body == "Infinity" {
        return Some(sign * f64::INFINITY);
    }
    if body.is_empty()
        || !body
            .bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'.' | b'e' | b'E' | b'+' | b'-'))
        || !body.bytes().any(|b| b.is_ascii_digit())
    {
        return None;
    }
    t.parse::<f64>().ok()
}

/// JavaScript's `text.slice(0, units)`: the first `units` UTF-16 units, but never
/// cutting a character in two (a character that would not fit whole is left out).
pub fn js_slice_units(text: &str, units: usize) -> String {
    let mut used = 0;
    let mut out = String::new();
    for c in text.chars() {
        used += c.len_utf16();
        if used > units {
            break;
        }
        out.push(c);
    }
    out
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

    #[test]
    fn js_to_fixed_other_digits() {
        assert_eq!(js_to_fixed(0.5, 0), "1");
        assert_eq!(js_to_fixed(1.5, 0), "2");
        assert_eq!(js_to_fixed(2.5, 0), "3");
        assert_eq!(js_to_fixed(-2.5, 0), "-3");
        assert_eq!(js_to_fixed(0.0, 0), "0");
        assert_eq!(js_to_fixed(0.0049, 4), "0.0049");
        assert_eq!(js_to_fixed(1.00005, 4), "1.0001");
        assert_eq!(js_to_fixed(0.0000005, 6), "0.000000");
        assert_eq!(js_to_fixed(-0.0, 6), "0.000000");
        assert_eq!(js_to_fixed(123.456, 2), "123.46");
    }

    #[test]
    fn js_number_to_string_matches_javascript() {
        assert_eq!(js_number_to_string(20.0), "20");
        assert_eq!(js_number_to_string(1.5), "1.5");
        assert_eq!(js_number_to_string(-0.0), "0");
        assert_eq!(js_number_to_string(0.1 + 0.2), "0.30000000000000004");
    }
}
