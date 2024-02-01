use clap::Parser;
use juliafatou::*;
use juliafatou::ColorStyle;
use std::sync::Arc;
use std::thread::available_parallelism;
use std::time::Instant;

#[derive(Parser, Debug)]
#[clap(author, version, about="render julia sets blazingly fast")]

struct Arguments {

    #[clap(short, long="dimensions", default_value="1200x1200", value_name="USIZExUSIZE")]
    /// Image dimensions
    dimensions: String,

    #[clap(short, long="output-file", default_value="output.png", value_name="FILE")]
    /// Output file
    out: String,

    #[clap(long, value_name="FILE")]
    /// custom color gradient
    config: Option<String>,

    #[clap(short='s', long="offset", default_value="0.0:0.0", allow_hyphen_values=true, value_name="F64:F64")]
    /// offset
    off: String,

    #[clap(short='x', long="scale", default_value_t=3.0, value_name="F64")]
    /// scale factor
    scale: f64,

    #[clap(long, default_value_t=1.0, value_name="F32")]
    /// blur (sigma)
    blur: f32,

    #[clap(short='w', long="power", default_value_t=2, value_name="U8")]
    /// the 'x' in the equation z^x + c
    power: u8,

    #[clap(short, long, default_value_t=-0.25, allow_hyphen_values=true, value_name="F64")]
    /// multiplication factor of the secondary julia set (intensity)
    factor: f64,

    #[clap(short='c', long="color-style", value_enum, default_value="greyscale")]
    /// Select color gradient
    cm: ColorStyle,

    #[clap(short='g', long="diverge", value_name="F64", default_value_t=0.01, allow_hyphen_values=true)]
    /// difference between the two rendered julia sets 
    gap: f64,

    #[clap(short='p', long="complex", value_name="F64,F64", default_value="-0.4,0.6", allow_hyphen_values=true)]
    /// the 'c' in the equation z^x + c
    complex: String,

    #[clap(short, long, value_name="F64", default_value_t=3.0)]
    /// overall intensity multiplication factor
    intensity: f64,

    #[clap(long, default_value_t=false)]
    /// invert color gradient
    inverse: bool,

    #[clap(long, value_name="USIZE")]
    /// number of threads (optional), defaults to 'available parallelism'
    threads: Option<usize>,

    #[clap(long, default_value_t=false)]
    /// measure render time
    take_time: bool,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    // parse arguments
    let args = Arguments::parse();

    // parse complex number
    let complex = parse_complex_number(&args.complex).expect("error parsing complex number");

    // parse image dimensions
    let dimensions: (usize, usize) = parse_values(&args.dimensions, 'x').expect("error parsing image dimensions");

    // scalex is used for both x and y axis in order to mitigate image distortion
    let scalex = args.scale / dimensions.1 as f64;
    
    // get x/y ratio of the image dimensions
    let ratio = dimensions.0 as f64 / dimensions.1 as f64;
    
    // parse offset
    let parsed_offset: (f64, f64) = parse_values(&args.off, ':').expect("error parsing offset"); 
    
    // calculate actual offset in a way that '0:0' will always result in a centered image
    let off = args.scale / 2.0;
    let offset = ((parsed_offset.0 - off) + off * ratio, parsed_offset.1, off);

    // initialize image buffer
    let mut pixels = vec![0u8; dimensions.0 * dimensions.1 * 3];

    // determine number of threads
    let threads = match args.threads {
        Some(value) => value,
        None => available_parallelism()?.get()
    };
    eprintln!("Using {} threads.", threads);
    
    // determine maximum number of pixel rows per thread
    let rows_per_band = dimensions.1 / threads + 1;

    // get the colors that are used to build the color gradient
    let color_array = return_colors(&args.cm, args.config);

    // build color gradient
    let grad = colorgrad::CustomGradient::new()
        .colors(&color_array)
        .domain(&[0.0, 255.0])
        .mode(colorgrad::BlendMode::Rgb)
        .build()?;

    // initialize atomic reference counting for the color gradient
    // in order to be shared safely between threads
    let grad_arc = Arc::new(grad);

    // initialize time measurement with the None variant
    let mut begin: Option<Instant> = None;

    // take start time if flag has been set
    if args.take_time {
        begin = Some(Instant::now());
    }

    // initialize scoped multithreading
    // every thread must know it's bounds and position in the overall image,
    // as well as the information that defines the corresponding julia set
    {
        let bands: Vec<&mut [u8]> = 
            
            pixels.chunks_mut(rows_per_band * dimensions.0 * 3).collect();
        
        crossbeam::scope(|spawner| {
            
            for (i, band) in bands.into_iter().enumerate() {
                
                let top = rows_per_band * i;
                
                let height = band.len() / dimensions.0 / 3;
                
                let band_upper_left = (0, top);

                let band_bounds = (dimensions.0, height);
                
                let cloned_arc = Arc::clone(&grad_arc);

                spawner.spawn(move |_| {
                        render(band,
                               band_bounds,
                               band_upper_left,
                               (scalex, scalex),
                               offset,
                               complex,
                               args.gap,
                               &cloned_arc,
                               args.intensity,
                               args.inverse,
                               args.power as u32,
                               args.factor);
                });
            }
        }).unwrap();
    }

    // minimalistic post processing
    blur_image(&args.out, &pixels, dimensions, args.blur).expect("error while blurring or writing the image");

    // take end time if flag has been set and print the duration
    if args.take_time {
        let duration = begin.unwrap().elapsed();
        eprintln!("time elapsed: {:?}", duration);
    }

    Ok(())
}
