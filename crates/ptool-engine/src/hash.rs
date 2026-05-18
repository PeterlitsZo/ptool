use std::io::Cursor;

use blake2::{Blake2b512, Blake2s256, digest::Digest as BlakeDigest};
use crc::{CRC_32_ISO_HDLC, CRC_64_ECMA_182, Crc};
use sha1::{Digest as _, Sha1};
use sha2::{Sha224, Sha256, Sha384, Sha512, Sha512_224, Sha512_256};
use sha3::{Sha3_224, Sha3_256, Sha3_384, Sha3_512};
use xxhash_rust::{
    xxh3::{xxh3_64, xxh3_128},
    xxh32::xxh32,
    xxh64::xxh64,
};

const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
const CRC64: Crc<u64> = Crc::<u64>::new(&CRC_64_ECMA_182);
const FNV1A32_OFFSET_BASIS: u32 = 0x811c9dc5;
const FNV1A32_PRIME: u32 = 0x0100_0193;
const FNV1A64_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV1A64_PRIME: u64 = 0x0000_0100_0000_01b3;

pub(crate) fn sha224_hex(bytes: &[u8]) -> String {
    digest_hex(Sha224::digest(bytes))
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    digest_hex(Sha256::digest(bytes))
}

pub(crate) fn sha384_hex(bytes: &[u8]) -> String {
    digest_hex(Sha384::digest(bytes))
}

pub(crate) fn sha512_hex(bytes: &[u8]) -> String {
    digest_hex(Sha512::digest(bytes))
}

pub(crate) fn sha512_224_hex(bytes: &[u8]) -> String {
    digest_hex(Sha512_224::digest(bytes))
}

pub(crate) fn sha512_256_hex(bytes: &[u8]) -> String {
    digest_hex(Sha512_256::digest(bytes))
}

pub(crate) fn sha1_hex(bytes: &[u8]) -> String {
    digest_hex(Sha1::digest(bytes))
}

pub(crate) fn sha3_224_hex(bytes: &[u8]) -> String {
    digest_hex(Sha3_224::digest(bytes))
}

pub(crate) fn sha3_256_hex(bytes: &[u8]) -> String {
    digest_hex(Sha3_256::digest(bytes))
}

pub(crate) fn sha3_384_hex(bytes: &[u8]) -> String {
    digest_hex(Sha3_384::digest(bytes))
}

pub(crate) fn sha3_512_hex(bytes: &[u8]) -> String {
    digest_hex(Sha3_512::digest(bytes))
}

pub(crate) fn blake2s256_hex(bytes: &[u8]) -> String {
    let mut hasher = Blake2s256::new();
    BlakeDigest::update(&mut hasher, bytes);
    digest_hex(BlakeDigest::finalize(hasher))
}

pub(crate) fn blake2b512_hex(bytes: &[u8]) -> String {
    let mut hasher = Blake2b512::new();
    BlakeDigest::update(&mut hasher, bytes);
    digest_hex(BlakeDigest::finalize(hasher))
}

pub(crate) fn blake3_hex(bytes: &[u8]) -> String {
    digest_hex(blake3::hash(bytes).as_bytes())
}

pub(crate) fn md5_hex(bytes: &[u8]) -> String {
    let digest = md5::compute(bytes);
    format!("{digest:x}")
}

pub(crate) fn crc32_hex(bytes: &[u8]) -> String {
    hex_u32(CRC32.checksum(bytes))
}

pub(crate) fn crc64_hex(bytes: &[u8]) -> String {
    hex_u64(CRC64.checksum(bytes))
}

pub(crate) fn adler32_hex(bytes: &[u8]) -> String {
    hex_u32(adler2::adler32_slice(bytes))
}

pub(crate) fn xxh32_hex(bytes: &[u8]) -> String {
    hex_u32(xxh32(bytes, 0))
}

pub(crate) fn xxh64_hex(bytes: &[u8]) -> String {
    hex_u64(xxh64(bytes, 0))
}

pub(crate) fn xxh3_64_hex(bytes: &[u8]) -> String {
    hex_u64(xxh3_64(bytes))
}

pub(crate) fn xxh3_128_hex(bytes: &[u8]) -> String {
    hex_u128(xxh3_128(bytes))
}

pub(crate) fn murmur3_32_hex(bytes: &[u8]) -> String {
    let mut cursor = Cursor::new(bytes);
    let digest =
        murmur3::murmur3_32(&mut cursor, 0).expect("hashing an in-memory buffer should not fail");
    hex_u32(digest)
}

pub(crate) fn murmur3_128_hex(bytes: &[u8]) -> String {
    let mut cursor = Cursor::new(bytes);
    let digest = murmur3::murmur3_x64_128(&mut cursor, 0)
        .expect("hashing an in-memory buffer should not fail");
    hex_u128(digest)
}

pub(crate) fn fnv1a32_hex(bytes: &[u8]) -> String {
    let mut hash = FNV1A32_OFFSET_BASIS;
    for &byte in bytes {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(FNV1A32_PRIME);
    }
    hex_u32(hash)
}

pub(crate) fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut hash = FNV1A64_OFFSET_BASIS;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV1A64_PRIME);
    }
    hex_u64(hash)
}

fn digest_hex(bytes: impl AsRef<[u8]>) -> String {
    hex_encode(bytes.as_ref())
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

fn hex_u32(value: u32) -> String {
    format!("{value:08x}")
}

fn hex_u64(value: u64) -> String {
    format!("{value:016x}")
}

fn hex_u128(value: u128) -> String {
    format!("{value:032x}")
}
