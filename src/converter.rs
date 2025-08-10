use std::string::FromUtf8Error;

pub struct SimpleConverter {}

impl SimpleConverter {
    pub fn new() -> SimpleConverter {
        SimpleConverter {}
    }
}

impl Converter for SimpleConverter {
    fn to_binary(&self, value: &str) -> Vec<bool> {

        let mut binary_digits = Vec::new();

        for byte in value.as_bytes() {
            for bit in (0u8..8).rev() {
                binary_digits.push(((byte >> bit) & 1) == 1);
            }
        }

        binary_digits
    }

    fn to_string(&self, bits: &[bool]) -> Result<String, FromUtf8Error> {
        let mut bytes = Vec::new();

        for chunk in bits.chunks(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    byte |= 1 << (7 - i);
                }
            }
            bytes.push(byte);
        }

        
        String::from_utf8(bytes)
    }
    
}

pub trait Converter {
    
    fn to_binary(&self, value: &str) -> Vec<bool>;
    fn to_string(&self, bits: &[bool]) -> Result<String, FromUtf8Error>;
}
