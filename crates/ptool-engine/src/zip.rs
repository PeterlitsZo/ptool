use crate::{Error, ErrorKind, Result};
use bzip2::{Compression as Bzip2Compression, read::BzDecoder, write::BzEncoder};
use flate2::{
    Compression as FlateCompression,
    read::{DeflateDecoder, GzDecoder, ZlibDecoder},
    write::{DeflateEncoder, GzEncoder, ZlibEncoder},
};
use std::io::{Cursor, Read, Write};
use xz2::{read::XzDecoder, write::XzEncoder};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZipFormat {
    Gzip,
    Zlib,
    Deflate,
    Bzip2,
    Xz,
    Zstd,
}

impl ZipFormat {
    pub fn parse(name: &str, op: &str) -> Result<Self> {
        match name {
            "gzip" | "gz" => Ok(Self::Gzip),
            "zlib" => Ok(Self::Zlib),
            "deflate" => Ok(Self::Deflate),
            "bzip2" | "bz2" => Ok(Self::Bzip2),
            "xz" => Ok(Self::Xz),
            "zstd" | "zst" | "zstandard" => Ok(Self::Zstd),
            _ => Err(Error::new(
                ErrorKind::Unsupported,
                format!("{op} does not support format `{name}`"),
            )
            .with_op(op)
            .with_input(name.to_string())),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Gzip => "gzip",
            Self::Zlib => "zlib",
            Self::Deflate => "deflate",
            Self::Bzip2 => "bzip2",
            Self::Xz => "xz",
            Self::Zstd => "zstd",
        }
    }
}

pub(crate) fn compress(bytes: &[u8], format: ZipFormat, op: &str) -> Result<Vec<u8>> {
    match format {
        ZipFormat::Gzip => compress_with_writer(
            bytes,
            GzEncoder::new(Vec::new(), FlateCompression::default()),
            format,
            op,
        ),
        ZipFormat::Zlib => compress_with_writer(
            bytes,
            ZlibEncoder::new(Vec::new(), FlateCompression::default()),
            format,
            op,
        ),
        ZipFormat::Deflate => compress_with_writer(
            bytes,
            DeflateEncoder::new(Vec::new(), FlateCompression::default()),
            format,
            op,
        ),
        ZipFormat::Bzip2 => compress_with_writer(
            bytes,
            BzEncoder::new(Vec::new(), Bzip2Compression::default()),
            format,
            op,
        ),
        ZipFormat::Xz => compress_with_writer(bytes, XzEncoder::new(Vec::new(), 6), format, op),
        ZipFormat::Zstd => zstd::stream::encode_all(Cursor::new(bytes), 0)
            .map_err(|err| compression_error("compression", format, op, err)),
    }
}

pub(crate) fn decompress(bytes: &[u8], format: ZipFormat, op: &str) -> Result<Vec<u8>> {
    match format {
        ZipFormat::Gzip => decompress_with_reader(GzDecoder::new(Cursor::new(bytes)), format, op),
        ZipFormat::Zlib => decompress_with_reader(ZlibDecoder::new(Cursor::new(bytes)), format, op),
        ZipFormat::Deflate => {
            decompress_with_reader(DeflateDecoder::new(Cursor::new(bytes)), format, op)
        }
        ZipFormat::Bzip2 => decompress_with_reader(BzDecoder::new(Cursor::new(bytes)), format, op),
        ZipFormat::Xz => decompress_with_reader(XzDecoder::new(Cursor::new(bytes)), format, op),
        ZipFormat::Zstd => zstd::stream::decode_all(Cursor::new(bytes))
            .map_err(|err| compression_error("decompression", format, op, err)),
    }
}

fn compress_with_writer<W>(
    bytes: &[u8],
    mut writer: W,
    format: ZipFormat,
    op: &str,
) -> Result<Vec<u8>>
where
    W: Write + Finish<Vec<u8>>,
{
    writer
        .write_all(bytes)
        .map_err(|err| compression_error("compression", format, op, err))?;
    writer
        .finish()
        .map_err(|err| compression_error("compression", format, op, err))
}

fn decompress_with_reader<R>(mut reader: R, format: ZipFormat, op: &str) -> Result<Vec<u8>>
where
    R: Read,
{
    let mut output = Vec::new();
    reader
        .read_to_end(&mut output)
        .map_err(|err| compression_error("decompression", format, op, err))?;
    Ok(output)
}

fn compression_error(
    action: &str,
    format: ZipFormat,
    op: &str,
    err: impl std::fmt::Display,
) -> Error {
    Error::new(
        ErrorKind::InvalidArgs,
        format!("{} {action} failed: {err}", format.name()),
    )
    .with_op(op)
}

trait Finish<T> {
    fn finish(self) -> std::io::Result<T>;
}

impl Finish<Vec<u8>> for GzEncoder<Vec<u8>> {
    fn finish(self) -> std::io::Result<Vec<u8>> {
        self.finish()
    }
}

impl Finish<Vec<u8>> for ZlibEncoder<Vec<u8>> {
    fn finish(self) -> std::io::Result<Vec<u8>> {
        self.finish()
    }
}

impl Finish<Vec<u8>> for DeflateEncoder<Vec<u8>> {
    fn finish(self) -> std::io::Result<Vec<u8>> {
        self.finish()
    }
}

impl Finish<Vec<u8>> for BzEncoder<Vec<u8>> {
    fn finish(self) -> std::io::Result<Vec<u8>> {
        self.finish()
    }
}

impl Finish<Vec<u8>> for XzEncoder<Vec<u8>> {
    fn finish(self) -> std::io::Result<Vec<u8>> {
        self.finish()
    }
}
