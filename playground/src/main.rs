use std::io::{Read, Write};
use brotli::{CompressorWriter, Decompressor};

pub trait Draw {
    fn draw(&self);
}


pub struct Screen<T: Draw> {
    pub components: Vec<T>,
}

impl<T> Screen<T>
where
    T: Draw,
{
    pub fn run(&self) {
        for component in self.components.iter() {
            component.draw();
        }
    }
}


fn main() {

}

fn compress() {
    // Define some data to compress
    let data = "Hello, world!".as_bytes();

    // Compress the data
    let mut compressed_data = Vec::new();
    {
        let mut compressor = CompressorWriter::new(&mut compressed_data, 4096, 11, 22);
        compressor.write_all(data).unwrap();
    }

    // Decompress the data
    let mut decompressed_data = Vec::new();
    {
        let mut decompressor = Decompressor::new(&compressed_data[..], 4096);
        decompressor.read_to_end(&mut decompressed_data).unwrap();
    }

    // Print the origin
    // Decompress the data
    let mut decompressed_data = Vec::new();
    {
        let mut decompressor = Decompressor::new(&compressed_data[..], 4096);
        decompressor.read_to_end(&mut decompressed_data).unwrap();
    }

    // Print the original and decompressed data
    let original_str = String::from_utf8_lossy(data);
    let decompressed_str = String::from_utf8_lossy(&decompressed_data);
    println!("Original data: {}", original_str);
    println!("Decompressed data: {}", decompressed_str);
}

