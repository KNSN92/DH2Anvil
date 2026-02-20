use std::io::{ErrorKind, Read};

use anyhow::{Result, anyhow};
use lz4_flex::frame::FrameDecoder;
use xz2::bufread::XzDecoder;

#[derive(Debug)]
pub enum CompressionMode {
    Uncompressed = 0,
    Lz4 = 1,
    ZStd = 2,
    Lzma2 = 3,
}

impl CompressionMode {
    pub fn from_num(n: usize) -> Option<CompressionMode> {
        match n {
            0 => Some(CompressionMode::Uncompressed),
            1 => Some(CompressionMode::Lz4),
            2 => Some(CompressionMode::ZStd),
            3 => Some(CompressionMode::Lzma2),
            _ => None,
        }
    }

    pub fn decompress(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        match self {
            CompressionMode::Uncompressed => Ok(data),
            CompressionMode::Lz4 => decompress_lz4(data),
            CompressionMode::ZStd => decompress_zstd(data),
            CompressionMode::Lzma2 => decompress_lzma2(data),
        }
    }
}

fn decompress_lz4(data: Vec<u8>) -> Result<Vec<u8>> {
    let mut decoder = FrameDecoder::new(data.as_slice());
    let mut decomped = Vec::new();
    decoder.read_to_end(&mut decomped)?;
    Ok(decomped)
}

fn decompress_zstd(data: Vec<u8>) -> Result<Vec<u8>> {
    let mut decoder = zstd::stream::read::Decoder::new(data.as_slice())?;
    let mut decomped = Vec::new();
    decoder.read_to_end(&mut decomped)?;
    Ok(decomped)
}

fn decompress_lzma2(data: Vec<u8>) -> Result<Vec<u8>> {
    let mut decoder = XzDecoder::new(data.as_slice());
    let mut decomped = Vec::new();
    let err = loop {
        let mut buf = [0u8; 1];
        if let Err(e) = decoder.read(&mut buf) {
            break e;
        }
        decomped.push(buf[0]);
    };
    if err.kind() == ErrorKind::UnexpectedEof {
        Ok(decomped)
    } else {
        Err(anyhow!(err))
    }
}
