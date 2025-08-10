use clap::ValueEnum;
use image::{GenericImageView, ImageReader, Rgba};
use rand::{Rng, SeedableRng};
use crate::converter::{Converter, SimpleConverter};
use sha2::{Sha256, Digest};
use rand_chacha::ChaCha20Rng;
use crate::steganography::EncodingLimit::U16;

fn string_to_seed_32(s: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    let result = hasher.finalize();
    result.into()
}

struct Traverser {
    random: ChaCha20Rng,
    area: Vec<usize>,
    iteration: usize,
    dimensions: (u32, u32),
}

impl Traverser {

    fn new(dimensions: (u32, u32), key: Option<String>) -> Traverser {

        let key = match key {
            None => {""}
            Some(x) => &*{ x }
        };

        let seed = string_to_seed_32(key);



        let mut area = Vec::with_capacity(dimensions.0 as usize * dimensions.1 as usize);
        for i in 0..(dimensions.0 * dimensions.1) {
            area.push(i as usize);
        }

        Traverser{
            area,
            random: ChaCha20Rng::from_seed(seed),
            iteration: 0,
            dimensions
        }

    }

    pub fn next(&mut self) -> Option<(u32, u32)> {
        if self.area.is_empty() {
            return None;
        }

        let index = self.random.random_range(0..self.area.len());
        let value = self.area.remove(index) as u32;

        self.iteration += 1;
        Some((value % self.dimensions.0, value / self.dimensions.0))
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum EncodingLimit {
    U8,
    U16,
    U32,
}

impl EncodingLimit {
    fn max(&self) -> usize {
        match self {
            EncodingLimit::U8 => u8::MAX as usize,
            U16 => u16::MAX as usize,
            EncodingLimit::U32 => u32::MAX as usize,
        }
    }

    fn bits(&self) -> usize {
        match self {
            EncodingLimit::U8 => 8usize,
            U16 => 16usize,
            EncodingLimit::U32 => 32usize,
        }
    }

    fn to_bool_vec(&self, number: u32) -> Vec<bool> {
        let bits = self.bits();
        (0..bits)
            .rev()
            .map(|i| ((number >> i) & 1) == 1)
            .collect()
    }
}

fn adjust_color(mut color: u8, value: bool) -> u8 {
    let is_even = color % 2 == 0;
    if value && is_even {
        color = color + 1;
    }
    else if !value && !is_even {
        color = color - 1;
    }

    color
}

fn value_in_pixel(pixel: Rgba<u8>) -> bool {
    let mut color = pixel[3];

    if pixel[3] == 0 {
        color = pixel[0]
    }

    color % 2 != 0
}


pub struct Steganography {
    key: Option<String>,
    converter: Box<dyn Converter>,
    encoding: EncodingLimit,
}

impl Steganography {
    pub fn new(key: Option<String>, encoding: Option<EncodingLimit>) -> Steganography {
        Steganography {
            key,
            converter: Box::new(SimpleConverter::new()),
            encoding: encoding.unwrap_or(U16)
        }
    }

    pub fn encode(&self, filename: &str, value: &str, output: Option<String>, verbose: bool) {
        let image = ImageReader::open(filename).unwrap().decode().unwrap();
        let dimensions = image.dimensions();

        let image_len = dimensions.0 * dimensions.1;

        if image_len == 0 {
            panic!("{}", format!("No image found at '{}'", filename));
        }

        let mut value_binary = self.converter.to_binary(value);

        if value_binary.is_empty() {
            panic!("{}", format!("No value binary found at '{}'", filename));
        }

        if value_binary.len() > self.encoding.max() {
            panic!("{}", format!("Value binary found at '{}'", filename));
        }

        let binary_encoding = self.encoding.to_bool_vec(value_binary.len() as u32);

        value_binary.splice(0..0, binary_encoding);

        let value_size =  value_binary.len();

        if value_size > image_len as usize {
            panic!("{}", format!("Value length is {} but the image can only hold {}"
                                 , value_size, image_len));
        }

        let mut traverser = Traverser::new(dimensions, self.key.clone());

        let mut rgba_image = image.to_rgba8();


        for value in value_binary {
            let pixel_pos = traverser.next().unwrap();
            let pixel = rgba_image.get_pixel(pixel_pos.0, pixel_pos.1).clone();
            let mut pixel = pixel;

            if pixel[3] > 1 {
                pixel[3] = adjust_color(pixel[3], value);
            } else {
                pixel[0] = adjust_color(pixel[0], value);
            }

            rgba_image.put_pixel(pixel_pos.0, pixel_pos.1, pixel);
        }

        let output_file = output.unwrap_or("output.png".to_string());

        if verbose {
            println!(
                "Output file: {}\n\
                Total Size: {}\n\
                Size Used: {} ( {:.2}% )\n\
                Encoding: {:?}",
                output_file,
                image_len,
                value_size,
                (value_size as f32 / image_len as f32) * 100.0,
                self.encoding
            );
        }

        rgba_image.save(output_file).unwrap();

    }

    pub fn decode(&self, filename: &str) -> Result<String, String> {
        let image = ImageReader::open(filename).unwrap().decode().unwrap();
        let dimensions = image.dimensions();

        let image_len = dimensions.0 * dimensions.1;

        if image_len == 0 {
            return Err(format!("No image found at '{}'", filename));
        }

        let mut traverser = Traverser::new(dimensions, self.key.clone());
        let mut bits_used = 0usize;

        for index in 0..self.encoding.bits() {
            let pixel_pos = traverser.next().unwrap();
            let value = value_in_pixel(image.get_pixel(pixel_pos.0, pixel_pos.1));
            if value {
                bits_used +=  2usize.pow((self.encoding.bits() - index - 1) as u32);
            }
        }

        if bits_used + self.encoding.bits() > image_len as usize {
            return Err("Error decoding image".to_owned());
        }

        let mut decoded = Vec::<bool>::with_capacity(bits_used);

        for _ in 0..bits_used {
            let pixel_pos = traverser.next().unwrap();
            let value = value_in_pixel(image.get_pixel(pixel_pos.0, pixel_pos.1));
            decoded.push(value);
        }


        self.converter.to_string(decoded.as_slice()).map_err(|e| e.to_string())
    }
}