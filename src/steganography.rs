use std::path::Path;
use clap::ValueEnum;
use image::{GenericImageView, ImageBuffer, ImageReader, Pixel, Rgba};
use crate::converter::{Converter, SimpleConverter};
use crate::steganography::EncodingLimit::B16;
use crate::traverser::Traverser;

const MAP_INTENSITY: u8 = 175;

#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum EncodingLimit {
    B8,
    B16,
    B32,
}


impl EncodingLimit {
    fn max(&self) -> usize {
        match self {
            EncodingLimit::B8 => u8::MAX as usize,
            EncodingLimit::B16 => u16::MAX as usize,
            EncodingLimit::B32 => u32::MAX as usize,
        }
    }

    fn bits(&self) -> usize {
        match self {
            EncodingLimit::B8 => 8usize,
            EncodingLimit::B16 => 16usize,
            EncodingLimit::B32 => 32usize,
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

#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum ColorChannel {
    RGB,
    RGBA
}

impl ColorChannel {
    fn get(&self) -> Vec<u8> {
        match self {
            ColorChannel::RGB => { vec![0, 1, 2] }
            ColorChannel::RGBA => { vec![0, 1, 2, 3] }
        }
    }
    
    fn color_count(&self) -> u32 {
        match self {
            ColorChannel::RGB => {3}
            ColorChannel::RGBA => {4}
        }
    }
}

fn adjust_color(color: u8, value: bool) -> u8 {
    let is_even = color % 2 == 0;

    if (value && is_even) || (!value && !is_even) {
        if color == 255 {
            color - 1
        } else {
            color + 1
        }
    } else {
        color
    }
}

fn value_in_pixel(pixel: Rgba<u8>, color: u8) -> bool {
    pixel[color.into()] % 2 != 0
}

fn image_capacity(dimensions: (u32, u32), color_count: u32) -> u32 {
    dimensions.0 * dimensions.1 * color_count
}


pub struct Steganography {
    key: Option<String>,
    converter: Box<dyn Converter>,
    encoding: EncodingLimit,
    color_channel: ColorChannel
}

impl Steganography {
    pub fn new(key: Option<String>, encoding: Option<EncodingLimit>, color_channel: Option<ColorChannel>) -> Steganography {
        Steganography {
            key,
            converter: Box::new(SimpleConverter::new()),
            encoding: encoding.unwrap_or(B16),
            color_channel: color_channel.unwrap_or(ColorChannel::RGBA)
        }
    }

    pub fn encode(&self, filename: &str, value: &str, output: Option<String>, verbose: bool, map: bool) {
        let image = ImageReader::open(filename).unwrap().decode().unwrap();
        let dimensions = image.dimensions();

        let image_len = image_capacity(dimensions, self.color_channel.color_count());

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

        let mut traverser = Traverser::new((dimensions.0, dimensions.1, self.color_channel.get()), self.key.clone());

        let mut rgba_image = image.to_rgba8();

        let mut map_image: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_fn(dimensions.0, dimensions.1, |_x, _y| Rgba([0, 0, 0, 255]));

        for value in value_binary {
            let pixel_pos = traverser.next().unwrap();
            let pixel = rgba_image.get_pixel(pixel_pos.0, pixel_pos.1).clone();
            let mut pixel = pixel;
            pixel[pixel_pos.2.into()] = adjust_color(pixel[pixel_pos.2.into()], value);
            rgba_image.put_pixel(pixel_pos.0, pixel_pos.1, pixel);
            
            if map {
                let mut map_pixel = map_image.get_pixel(pixel_pos.0, pixel_pos.1).to_rgba();
                map_pixel[pixel_pos.2.into()] = MAP_INTENSITY.abs_diff(map_pixel[pixel_pos.2.into()]);
                map_image.put_pixel(pixel_pos.0, pixel_pos.1, map_pixel);
            }
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
        
        rgba_image.save(output_file.clone()).unwrap();
        
        if map {
            let path = Path::new(&output_file);
            let mut new_name = path.file_stem().unwrap().to_os_string();
            new_name.push("_map");

            if let Some(ext) = path.extension() {
                new_name.push(".");
                new_name.push(ext);
            }

            let new_path = path.with_file_name(new_name);
            map_image.save(new_path).unwrap();
        }
    
    }

    pub fn decode(&self, filename: &str) -> Result<String, String> {
        let image = ImageReader::open(filename).unwrap().decode().unwrap();
        let dimensions = image.dimensions();

        let image_len = image_capacity(dimensions, self.color_channel.color_count());

        if image_len == 0 {
            return Err(format!("No image found at '{}'", filename));
        }

        let mut traverser = Traverser::new((dimensions.0, dimensions.1, self.color_channel.get()), self.key.clone());
        let mut bits_used = 0usize;
        let encoding_bits = self.encoding.bits();

        for index in 0..encoding_bits {
            let pixel_pos = traverser.next().unwrap();
            if value_in_pixel(image.get_pixel(pixel_pos.0, pixel_pos.1), pixel_pos.2) {
                bits_used += 1 << (encoding_bits - index - 1);
            }

        }

        if bits_used + encoding_bits > image_len as usize {
            return Err("Error decoding image".to_owned());
        }
    
        let mut decoded = Vec::<bool>::with_capacity(bits_used);

        for _ in 0..bits_used {
            let pixel_pos = traverser.next().unwrap();
            let value = value_in_pixel(image.get_pixel(pixel_pos.0, pixel_pos.1), pixel_pos.2);
            decoded.push(value);
        }


        self.converter.to_string(decoded.as_slice()).map_err(|e| e.to_string())
    }
}