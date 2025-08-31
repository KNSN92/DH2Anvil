use std::io::{ErrorKind, Read};

use anyhow::{Result, anyhow};
use xz2::bufread::XzDecoder;

#[derive(Debug)]
pub enum CompressionMode {
    Uncompressed = 0,
    Lz4 = 1,
    Lzma2 = 3,
}

impl CompressionMode {
    pub fn from_num(n: usize) -> Option<CompressionMode> {
        match n {
            0 => Some(CompressionMode::Uncompressed),
            1 => Some(CompressionMode::Lz4),
            3 => Some(CompressionMode::Lzma2),
            _ => None,
        }
    }

    pub fn decompress(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        match self {
            CompressionMode::Uncompressed => Ok(data),
            CompressionMode::Lz4 => unimplemented!("Lz4 Format is not implemented"),
            CompressionMode::Lzma2 => decompress_lzma2(data),
        }
    }
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
