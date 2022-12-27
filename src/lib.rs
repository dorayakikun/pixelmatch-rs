mod errors;

use crate::errors::Error as PixelMatchError;
use anyhow::Result;
use clap::{Arg, ArgAction, Command};
use image::DynamicImage;
use image::GenericImageView;
use image::ImageBuffer;
use image::Rgba;
use std::io::{self, Write};

pub fn run() -> anyhow::Result<()> {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("pixelmatch")
        .arg(
            Arg::new("threshold")
                .help("threshold")
                .long("threshold")
                .default_value("0.1")
                .value_parser(clap::value_parser!(f64)),
        )
        .arg(
            Arg::new("aa")
                .help("is include antialiased")
                .long("include-antialiased")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("dest")
                .help("destination path")
                .short('d')
                .long("dest")
                .default_value("-"),
        )
        .arg(Arg::new("before").index(1).required(true))
        .arg(Arg::new("after").index(2).required(true))
        .get_matches();

    let threshold = matches.get_one::<f64>("threshold").unwrap().to_owned();
    let is_include_aa = matches.get_flag("aa");

    let before = matches.get_one::<String>("before").unwrap();
    let after = matches.get_one::<String>("after").unwrap();
    let img1 = image::open(before)?;
    let img2 = image::open(after)?;

    let mut out = image::ImageBuffer::new(img1.width(), img1.height());

    let diff = match_pixel(&img1, &img2, &mut out, threshold, is_include_aa)?;

    if let Some(dest) = matches.get_one::<String>("dest") {
        match dest.as_str() {
            "-" => {
                io::stdout()
                    .write_all(&format!("diff: {}", diff).into_bytes())
                    .unwrap();
            }
            _ => {
                out.save(dest)?;
            }
        }
    }
    Ok(())
}

fn blend(c: u8, a: f64) -> u8 {
    (255. + f64::from(i32::from(c) - 255) * a) as u8
}

fn rgb2y(r: u8, g: u8, b: u8) -> f64 {
    f64::from(r) * 0.298_895_31 + f64::from(g) * 0.586_622_47 + f64::from(b) * 0.114_482_23
}

fn rgb2i(r: u8, g: u8, b: u8) -> f64 {
    f64::from(r) * 0.595_977_99 - f64::from(g) * 0.274_176_10 - f64::from(b) * 0.321_801_89
}

fn rgb2q(r: u8, g: u8, b: u8) -> f64 {
    f64::from(r) * 0.211_470_17 - f64::from(g) * 0.522_617_11 + f64::from(b) * 0.311_146_94
}

fn color_delta(pixel1: Rgba<u8>, pixel2: Rgba<u8>, y_only: bool) -> f64 {
    let mut r1 = pixel1[0];
    let mut g1 = pixel1[1];
    let mut b1 = pixel1[2];
    let a1 = pixel1[3];

    let mut r2 = pixel2[0];
    let mut g2 = pixel2[1];
    let mut b2 = pixel2[2];
    let a2 = pixel2[3];

    if r1 == r2 && g1 == g2 && b1 == b2 && a1 == a2 {
        return 0.;
    }

    if a1 < 255 {
        let a1 = f64::from(a1) / 255.; // alpha 0 ~ 1
        r1 = blend(r1, a1);
        g1 = blend(g1, a1);
        b1 = blend(b1, a1);
    }
    if a2 < 255 {
        let a2 = f64::from(a2) / 255.; // alpha 0 ~ 1
        r2 = blend(r2, a2);
        g2 = blend(g2, a2);
        b2 = blend(b2, a2);
    }

    let y = rgb2y(r1, g1, b1) - rgb2y(r2, g2, b2);

    if y_only {
        return y;
    }
    let i = rgb2i(r1, g1, b1) - rgb2i(r2, g2, b2);
    let q = rgb2q(r1, g1, b1) - rgb2q(r2, g2, b2);

    0.5053 * y * y + 0.299 * i * i + 0.1957 * q * q
}

fn is_antialiased(
    img1: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    img2: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: u32,
    y: u32,
) -> bool {
    let (_, _, iw, ih) = img1.bounds();
    if x == 0 || x == iw - 1 || y == 0 || y == ih - 1 {
        // when on the edge
        return false;
    }

    let mut zeroes: u32 = 0;
    let mut min: f64 = 0.;
    let mut max: f64 = 0.;
    let mut min_x: i32 = -1;
    let mut min_y: i32 = -1;
    let mut max_x: i32 = -1;
    let mut max_y: i32 = -1;

    let pixel = *img1.get_pixel(x, y);
    for dx in -1i32..=1 {
        for dy in -1i32..=1 {
            if dx == 0 && dy == 0 {
                // current pixel is origin
                continue;
            }

            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            let delta = color_delta(pixel, *img2.get_pixel(nx as u32, ny as u32), true);
            if delta == 0. {
                zeroes += 1;
                if zeroes > 2 {
                    return true;
                }
            } else if delta < min {
                min = delta;
                min_x = nx;
                min_y = ny;
            } else if delta > max {
                max = delta;
                max_x = nx;
                max_y = ny;
            }
        }
    }
    if max == 0. || min == 0. {
        return false;
    }
    (has_many_siblings(img1, min_x as u32, min_y as u32)
        && has_many_siblings(img2, min_x as u32, min_y as u32))
        || (has_many_siblings(img1, max_x as u32, max_y as u32)
            && has_many_siblings(img2, max_x as u32, max_y as u32))
}

fn has_many_siblings(img: &ImageBuffer<Rgba<u8>, Vec<u8>>, x: u32, y: u32) -> bool {
    let (_, _, iw, ih) = img.bounds();
    if x == 0 || x == iw - 1 || y == 0 || y == ih - 1 {
        // when on the edge
        return false;
    }

    let [r, g, b, a] = img.get_pixel(x, y).0;
    let mut zeroes: u32 = 0;
    for dx in -1i32..=1 {
        for dy in -1i32..=1 {
            if dx == 0 && dy == 0 {
                // current pixel is origin
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            let [nr, ng, nb, na] = img.get_pixel(nx as u32, ny as u32).0;
            if r == nr && g == ng && b == nb && a == na {
                zeroes += 1;
                if zeroes > 2 {
                    break;
                }
            }
        }
    }
    zeroes > 2
}

fn draw_pixel(out: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, x: u32, y: u32, r: u8, g: u8, b: u8) {
    out.put_pixel(x, y, Rgba([r, g, b, 255]))
}

fn gray_pixel(pixel: Rgba<u8>, alpha: f64) -> u8 {
    blend(
        rgb2y(pixel[0], pixel[1], pixel[2]) as u8,
        alpha * f64::from(pixel[3]) / 255.,
    )
}

pub fn match_pixel(
    img1: &DynamicImage,
    img2: &DynamicImage,
    out: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    threshold: f64,
    include_aa: bool,
) -> Result<u32> {
    if img1.dimensions() != img2.dimensions() {
        return Err(anyhow::Error::new(PixelMatchError::SizeUnmatch {
            before: img1.dimensions(),
            after: img2.dimensions(),
        }));
    }
    let max_delta = 35215. * threshold * threshold;
    let mut diff: u32 = 0;

    let img1 = img1.to_rgba8();
    let img2 = img2.to_rgba8();

    for (x, y, pixel1) in img1.enumerate_pixels() {
        let delta = color_delta(*pixel1, *img2.get_pixel(x, y), false);

        if delta > max_delta {
            if !include_aa
                && (is_antialiased(&img1, &img2, x, y) || is_antialiased(&img2, &img1, x, y))
            {
                draw_pixel(out, x, y, 255, 255, 0);
            } else {
                draw_pixel(out, x, y, 255, 0, 0);
                diff += 1;
            }
        } else {
            let val = gray_pixel(*pixel1, 0.1);
            draw_pixel(out, x, y, val, val, val)
        }
    }
    Ok(diff)
}
