use std::{fs, io};
use std::io::{Error, Read, Write};
use base64::{engine::general_purpose, Engine as _};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use flate2::read::ZlibDecoder;

pub fn file_to_b64(file_path: &str) -> Result<String, Error> {
    let bytes = fs::read(file_path)?;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&bytes)?;
    let compressed_bytes = encoder.finish()?;

    let b64 = general_purpose::STANDARD.encode(&compressed_bytes);

    Ok(b64)
}

pub fn b64_to_file(b64: &str, out_file_path: &str) -> Result<(), io::Error> {
    let compressed_bytes = general_purpose::STANDARD
        .decode(b64)
        .map_err(|e| Error::new(io::ErrorKind::InvalidData, e))?;

    let mut decoder = ZlibDecoder::new(&compressed_bytes[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    fs::write(out_file_path, decompressed)?;

    Ok(())
}