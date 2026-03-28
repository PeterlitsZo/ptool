use sha1::{Digest as _, Sha1};
use sha2::Sha256;

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex_encode(digest.as_slice())
}

pub(crate) fn sha1_hex(bytes: &[u8]) -> String {
    let digest = Sha1::digest(bytes);
    hex_encode(digest.as_slice())
}

pub(crate) fn md5_hex(bytes: &[u8]) -> String {
    let digest = md5::compute(bytes);
    format!("{digest:x}")
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX_DIGITS[(byte >> 4) as usize] as char);
        output.push(HEX_DIGITS[(byte & 0x0f) as usize] as char);
    }
    output
}
