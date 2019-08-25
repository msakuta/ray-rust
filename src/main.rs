extern crate image;

use std::fs::File;
use image::png::PNGEncoder;
use image::ColorType;

const WIDTH: usize = 64;
const HEIGHT: usize = 64;

fn main() -> std::io::Result<()> {
    println!("Hello, world!");

    let mut data: [u8; (WIDTH * HEIGHT)] = [0u8; WIDTH * HEIGHT];

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            data[x + y * WIDTH] = ((x + y) % 64) as u8;
        }
    }

    let buffer = File::create("foo.png")?;
    let encoder = PNGEncoder::new(buffer);

    encoder.encode(&data, WIDTH as u32, HEIGHT as u32, ColorType::Gray(8))
}
