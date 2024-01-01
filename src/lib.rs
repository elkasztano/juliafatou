use image::{ImageBuffer, Rgb};
use image::imageops::blur;
use num::Complex;
use std::str::FromStr;
use std::fs::read_to_string;
use clap::ValueEnum;
use colorgrad::Color;
use getrandom::getrandom;

// value enum for the command line argument parser

#[derive(ValueEnum,Copy,Clone,Debug)]
pub enum ColorStyle {
    Bookworm,
    Jellyfish,
    Ten,
    Eleven,
    Mint,
    Greyscale,
    Christmas,
    Chameleon,
    Plasma,
    Plasma2,
    Config,
    Random
}

// calculate offset for viewpoint

fn calculate_offset(pixel: (usize, usize), scale: (f64, f64), offset: (f64, f64, f64) ) -> Complex<f64> {
    
    let cx = pixel.1 as f64 * scale.1 - (offset.2 + offset.0);
    let cy = pixel.0 as f64 * scale.0 - (offset.2 + offset.1);

    Complex { re: cx, im: cy }

}

// get smoothly shaded colors

fn make_smooth(c: Complex<f64>, i: usize) -> f64 {

    i as f64 + 2.0 - ((c.re.powi(2) + c.im.powi(2)).ln()).ln() / std::f64::consts::LN_2

}

// plotting algorithm for the julia set

fn escape_time(pixel: (usize, usize), scale: (f64, f64), offset: (f64, f64, f64), c: Complex<f64>, limit: usize, power: u32) -> Option<f64> {

    let mut z = calculate_offset(pixel, scale, offset);
    
    for i in 0..limit {
        
        if z.norm_sqr() > 5.0 {
                
            return Some(make_smooth(z, i));
        
        } 
        
        z = z.powu(power) + c;
    }

    None
}

// parse two values of a certain type with a specified separator

pub fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None,
            }
        }
    }
}

// parse complex number

pub fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None,
    }
}

// calculate complex number for secundary julia set

fn get_diverged(c: Complex<f64>, diverge: f64) -> (Complex<f64>, Complex<f64>) {
    let altered = Complex { re: c.re + diverge, im: c.im - diverge };

    (c, altered)
}

// actual render function

pub fn render(pixels: &mut [u8],
          bounds: (usize, usize),
          upper_left: (usize, usize),
          scale: (f64, f64),
          offset: (f64, f64, f64),
          complex: Complex<f64>,
          diverge: f64,
          grad: &colorgrad::Gradient,
          intensity: f64,
          inverse: bool,
          power: u32,
          factor: f64,
          )
{
    assert!(pixels.len() == bounds.0 * bounds.1 * 3);

    for column in 0..bounds.0 {
        for row in 0..bounds.1 {
            let x = row + upper_left.1;
            let y = column + upper_left.0;
            let point = (x, y);

            let diverged = get_diverged(complex, diverge);

            let a = escape_time(point, scale, offset, diverged.0, 1024, power).unwrap_or(0.0);
            let b = escape_time(point, scale, offset, diverged.1, 1024, power).unwrap_or(0.0);
            
            let mut x = (a + b * factor) / (1.0 + factor);

            if inverse {
                x = 255.0 - x;
            }

            let newpix: [u8; 4] = grad.reflect_at(x * intensity).to_rgba8();

            for rgb in 0..3 {
                pixels[row * (bounds.0 * 3) + column * 3 + rgb] = newpix[rgb];
            }
        }
    }
}

// perform minimalistic post processing, save image buffer to file

pub fn blur_image(filename: &str, pixels: &[u8], bounds: (usize, usize), sigma: f32) -> Result<(), Box<dyn std::error::Error>> {

    let internalbuf: ImageBuffer<Rgb<u8>, &[u8]> = ImageBuffer::from_raw(bounds.0 as u32, bounds.1 as u32, pixels).unwrap();

    let blurred = blur(&internalbuf, sigma);

    blurred.save(filename)?;

    Ok(())

}

// write image buffer to file without post processing - currently unused
/*
pub fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize)) -> Result<(), Box<dyn std::error::Error>>
{
    
    let buf: ImageBuffer<Rgb<u8>, &[u8]> = ImageBuffer::from_raw(bounds.0 as u32, bounds.1 as u32, pixels).unwrap();

    buf.save(filename)?;

    Ok(())

}
*/

// return three colors for the color gradient

pub fn return_colors (style: &ColorStyle, path_opt: Option<String>) -> [Color; 3] {
        
    match style {
        ColorStyle::Bookworm => [
            Color::from_rgba8(5, 71, 92, 255),
            Color::from_rgba8(10, 120, 115, 255),
            Color::from_rgba8(184, 216, 215, 255)
        ],
        ColorStyle::Jellyfish => [
            Color::from_rgba8(38, 0, 24, 255),
            Color::from_rgba8(90, 25, 63, 255),
            Color::from_rgba8(198, 70, 72, 255)
        ],
        ColorStyle::Ten => [
            Color::from_rgba8(4, 62, 185, 255),
            Color::from_rgba8(2, 123, 230, 255),
            Color::from_rgba8(105, 254, 255, 255)
        ],
        ColorStyle::Greyscale => [
            Color::from_rgba8(255, 255, 255, 255),
            Color::from_rgba8(127, 127, 127, 255),
            Color::from_rgba8(0, 0, 0, 255)
        ],
        ColorStyle::Eleven => [
            Color::from_rgba8(2, 70, 217, 255),
            Color::from_rgba8(1, 214, 244, 255),
            Color::from_rgba8(209, 229, 254, 255),
        ],
        ColorStyle::Mint => [
            Color::from_rgba8(21, 21, 21, 255),
            Color::from_rgba8(137, 184, 70, 255),
            Color::from_rgba8(214, 214, 214, 255),
        ],
        ColorStyle::Chameleon => [
            Color::from_rgba8(11, 127, 109, 255),
            Color::from_rgba8(35, 145, 108, 255),
            Color::from_rgba8(21, 155, 110, 255),
        ],
        ColorStyle::Plasma => [
            Color::from_rgba8(35, 37, 83, 255),
            Color::from_rgba8(36, 102, 156, 255),
            Color::from_rgba8(219, 135, 75, 255),
        ],
        ColorStyle::Plasma2 => [
            Color::from_rgba8(0, 87, 139, 255),
            Color::from_rgba8(0, 147, 235, 255),
            Color::from_rgba8(249, 249, 249, 255),
        ],
        ColorStyle::Christmas => [
            Color::from_rgba8(31, 56, 35, 255),
            Color::from_rgba8(209, 27, 79, 255),
            Color::from_rgba8(250, 219, 82, 255),
        ],
        ColorStyle::Config => get_colors_from_file(path_opt).expect("error parsing colors from file"),
        ColorStyle::Random => get_random_colors().expect("error getting random colors"),
    }

}

// get three colors from csv file - basic attempt

fn get_colors_from_file(path_opt: Option<String>) -> Result<[Color; 3], Box<dyn std::error::Error>> {

    let filename = match path_opt {
        Some(path) => path,
        None => String::from("colors.csv"),
    };

    eprintln!("config file: '{}'", &filename);

    let mut output: [Color; 3] = [
        Color::from_rgba8(0,0,0,0),
        Color::from_rgba8(0,0,0,0),
        Color::from_rgba8(0,0,0,0)
    ];

    let strings: Vec<String> = read_to_string(filename)?.lines().skip(1).map(String::from).collect();

    assert!(strings.len() == 3);

    for string in strings.iter().enumerate() {

        let mut iterator = string.1.split(',');

            output[string.0] = Color::from_rgba8(
                iterator.next().unwrap_or("0").parse()?,
                iterator.next().unwrap_or("0").parse()?,
                iterator.next().unwrap_or("0").parse()?,
                255
            );

    }

    Ok(output)
}

// get three random colors

fn get_random_colors() -> Result<[Color; 3], Box<getrandom::Error>> {

    let mut random_data = [0u8; 9];

    getrandom(&mut random_data)?;

    eprintln!("R,G,B\n{},{},{}\n{},{},{}\n{},{},{}",
              random_data[0], random_data[1], random_data[2],
              random_data[3], random_data[4], random_data[5],
              random_data[6], random_data[7], random_data[8]);

    let output = [
        Color::from_rgba8(random_data[0], random_data[1], random_data[2], 255),
        Color::from_rgba8(random_data[3], random_data[4], random_data[5], 255),
        Color::from_rgba8(random_data[6], random_data[7], random_data[8], 255)
    ];

    Ok(output)
}
