//! Base64, decoded by hand.
//!
//! The sending proof travels as base64 inside JSON, and decoding it is the one
//! thing the legacy area needs that the `std` does not have. A whole crate for
//! one alphabet would be a fourth dependency on every tree that compiles this
//! SDK -- for sixty lines.

/// Decodes standard base64. `None` when the text is not valid base64.
///
/// Strict on purpose. A lenient decoder -- one that skips whatever it does not
/// recognise -- hands back plausible-looking bytes for an answer that was never
/// a PDF, and the caller writes a corrupt file to disk instead of learning that
/// the gateway said something else. Refusing here turns that into a typed
/// error one line up.
///
/// The one thing it does tolerate is whitespace, because a base64 field that
/// crossed a pretty printer comes back wrapped in newlines and is still the
/// same PDF.
pub(crate) fn decode(content: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::with_capacity(content.len() / 4 * 3);
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;
    let mut symbols: usize = 0;
    let mut padding: usize = 0;

    for symbol in content.bytes() {
        if symbol.is_ascii_whitespace() {
            continue;
        }

        if symbol == b'=' {
            padding += 1;
            continue;
        }

        // Data after the padding means the text was cut and glued back
        // together -- decoding it would hand over a corrupt PDF.
        if padding > 0 {
            return None;
        }

        buffer = (buffer << 6) | u32::from(value_of(symbol)?);
        bits += 6;
        symbols += 1;

        if bits >= 8 {
            bits -= 8;
            bytes.push(u8::try_from((buffer >> bits) & 0xFF).ok()?);
        }
    }

    // Six leftover bits is a group of one symbol, which encodes nothing.
    if bits == 6 {
        return None;
    }

    // Padding only ever completes the last group: at most two signs, and only
    // where the group was short. `QQ==` is a byte; `QQ=` and `QUJD====` are
    // someone else's text.
    if padding > 0 && (padding > 2 || (symbols + padding) % 4 != 0) {
        return None;
    }

    Some(bytes)
}

fn value_of(symbol: u8) -> Option<u8> {
    match symbol {
        b'A'..=b'Z' => Some(symbol - b'A'),
        b'a'..=b'z' => Some(symbol - b'a' + 26),
        b'0'..=b'9' => Some(symbol - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
