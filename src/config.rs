extern crate clap;

use image::ImageOutputFormat;
use std::f32;
use std::path::Path;

#[derive(Clone)]
pub struct Material {
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
}

#[derive(Clone)]
pub enum AAMethod {
    None,
    FXAA,
}

#[derive(Clone)]
pub struct Config {
    pub stl_filename: String,
    pub img_filename: String,
    pub format: ImageOutputFormat,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
    pub verbosity: usize,
    pub material: Material,
    pub background: (f32, f32, f32, f32),
    pub aamethod: AAMethod,
    pub recalc_normals: bool,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            stl_filename: "".to_string(),
            img_filename: "".to_string(),
            format: ImageOutputFormat::Png,
            width: 1024,
            height: 768,
            visible: false,
            verbosity: 0,
            material: Material {
                ambient: [0.00, 0.13, 0.26],
                diffuse: [0.38, 0.63, 1.00],
                specular: [1.00, 1.00, 1.00],
            },
            background: (0.0, 0.0, 0.0, 0.0),
            aamethod: AAMethod::FXAA,
            recalc_normals: false,
            x: 2.0,
            y: -4.0,
            z: 2.0
        }
    }
}

impl Config {
    pub fn new() -> Config {
        // Define command line arguments
        let matches = clap::Command::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .arg(
                clap::Arg::new("STL_FILE")
                    .help("STL file. Use - to read from stdin instead of a file.")
                    .required(true)
                    .index(1),
            )
            .arg(
                clap::Arg::new("IMG_FILE")
                    .help("Thumbnail image file. Use - to write to stdout instead of a file.")
                    .required(true)
                    .index(2),
            )
            .arg(
                clap::Arg::new("format")
                    .help("The format of the image file. If not specified it will be determined from the file extension, or default to PNG if there is no extension. Supported formats: PNG, JPEG, GIF, ICO, BMP")
                    .short('f')
                    .long("format")
                    .takes_value(true)
            )
            .arg(
                clap::Arg::new("size")
                    .help("Size of thumbnail (square)")
                    .short('s')
                    .long("size")
                    .takes_value(true)
                    .required(false)
            )
            .arg(
                clap::Arg::new("x")
                    .help("x")
                    .short('x')
                    .long("x")
                    .takes_value(true)
                    .required(false)
            )
            .arg(
                clap::Arg::new("y")
                    .help("y")
                    .short('y')
                    .long("y")
                    .takes_value(true)
                    .required(false)
            )
            .arg(
                clap::Arg::new("z")
                    .help("z")
                    .short('z')
                    .long("z")
                    .takes_value(true)
                    .required(false)
            )
            .arg(
                clap::Arg::new("visible")
                    .help("Display the thumbnail in a window instead of saving a file")
                    .short('w')
                    .required(false)
            )
            .arg(
                clap::Arg::new("verbosity")
                    .short('v')
                    .multiple_occurrences(true)
                    .help("Increase message verbosity")
            )
            .arg(
                clap::Arg::new("material")
                    .help("Colors for rendering the mesh using the Phong reflection model. Requires 3 colors as rgb hex values: ambient, diffuse, and specular. Defaults to blue.")
                    .short('m')
                    .long("material")
                    .value_names(&["ambient","diffuse","specular"])
            )
            .arg(
                clap::Arg::new("background")
                    .help("The background color with transparency (rgba). Default is ffffff00.")
                    .short('b')
                    .long("background")
                    .takes_value(true)
                    .required(false)
            )
            .arg(
                clap::Arg::new("aamethod")
                    .help("Anti-aliasing method. Default is FXAA, which is fast but may introduce artifacts.")
                    .short('a')
                    .long("antialiasing")
                    .possible_values(["none", "fxaa"]),
            )
            .arg(
                clap::Arg::new("recalc_normals")
                    .help("Force recalculation of face normals. Use when dealing with malformed STL files.")
                    .long("recalc-normals")
            )
            .get_matches();

        let mut c = Config {
            ..Default::default()
        };

        c.stl_filename = matches.value_of("STL_FILE").unwrap().to_string();
        c.img_filename = matches.value_of("IMG_FILE").unwrap().to_string();
        match matches.value_of("format") {
            Some(x) => c.format = match_format(x),
            None => match Path::new(&c.img_filename).extension() {
                Some(ext) => c.format = match_format(ext.to_str().unwrap()),
                _ => (),
            },
        };
        matches
            .value_of("size")
            .map(|x| c.width = x.parse::<u32>().expect("Invalid size"));
        matches
            .value_of("size")
            .map(|x| c.height = x.parse::<u32>().expect("Invalid size"));

        matches
            .value_of("x")
            .map(|x| c.x = x.parse::<f32>().expect("Invalid x"));

        matches
            .value_of("y")
            .map(|x| c.y = x.parse::<f32>().expect("Invalid y"));

        matches
            .value_of("z")
            .map(|x| c.z = x.parse::<f32>().expect("Invalid z"));


        c.visible = matches.is_present("visible");
        c.verbosity = matches.occurrences_of("verbosity") as usize;
        match matches.values_of("material") {
            Some(mut x) => {
                c.material = Material {
                    ambient: html_to_rgb(x.next().unwrap()),
                    diffuse: html_to_rgb(x.next().unwrap()),
                    specular: html_to_rgb(x.next().unwrap()),
                }
            }
            _ => (),
        };
        matches
            .value_of("background")
            .map(|x| c.background = html_to_rgba(x));
        match matches.value_of("aamethod") {
            Some(x) => match x {
                "none" => c.aamethod = AAMethod::None,
                "fxaa" => c.aamethod = AAMethod::FXAA,
                _ => unreachable!(),
            },
            _ => (),
        };
        c.recalc_normals = matches.is_present("recalc_normals");

        c
    }
}

fn match_format(ext: &str) -> ImageOutputFormat {
    match ext.to_lowercase().as_ref() {
        "png" => ImageOutputFormat::Png,
        "jpeg" => ImageOutputFormat::Jpeg(95),
        "jpg" => ImageOutputFormat::Jpeg(95),
        "gif" => ImageOutputFormat::Gif,
        "ico" => ImageOutputFormat::Ico,
        "bmp" => ImageOutputFormat::Bmp,
        _ => {
            warn!("Unsupported image format. Using PNG instead.");
            Config {
                ..Default::default()
            }
            .format
        }
    }
}

fn html_to_rgb(color: &str) -> [f32; 3] {
    let r: f32 = u8::from_str_radix(&color[0..2], 16).expect("Invalid color") as f32 / 255.0;
    let g: f32 = u8::from_str_radix(&color[2..4], 16).expect("Invalid color") as f32 / 255.0;
    let b: f32 = u8::from_str_radix(&color[4..6], 16).expect("Invalid color") as f32 / 255.0;
    [r, g, b]
}

fn html_to_rgba(color: &str) -> (f32, f32, f32, f32) {
    let r: f32 = u8::from_str_radix(&color[0..2], 16).expect("Invalid color") as f32 / 255.0;
    let g: f32 = u8::from_str_radix(&color[2..4], 16).expect("Invalid color") as f32 / 255.0;
    let b: f32 = u8::from_str_radix(&color[4..6], 16).expect("Invalid color") as f32 / 255.0;
    let a: f32 = u8::from_str_radix(&color[6..8], 16).expect("Invalid color") as f32 / 255.0;
    (r, g, b, a)
}
