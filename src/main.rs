mod errors;
mod pixelmatch;

use clap::{App, Arg};
use image::GenericImageView;
use std::io::{self, Write};

fn run() -> anyhow::Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("pixelmatch")
        .arg(
            Arg::new("threshold")
                .help("threshold")
                .long("threshold")
                .default_value("0.1")
                .takes_value(true),
        )
        .arg(
            Arg::new("aa")
                .help("is include antialiased")
                .long("include-antialiased"),
        )
        .arg(
            Arg::new("dest")
                .help("destination path")
                .short('d')
                .long("dest")
                .default_value("-")
                .takes_value(true),
        )
        .arg(Arg::new("before").index(1).required(true))
        .arg(Arg::new("after").index(2).required(true))
        .get_matches();

    let threshold: f64 = matches.value_of("threshold").unwrap().parse()?;
    let is_include_aa = matches.is_present("aa");

    let before = matches.value_of("before").unwrap();
    let after = matches.value_of("after").unwrap();
    let img1 = image::open(before)?;
    let img2 = image::open(after)?;

    let mut out = image::ImageBuffer::new(img1.width(), img1.height());

    let diff = pixelmatch::match_pixel(&img1, &img2, &mut out, threshold, is_include_aa)?;

    if let Some(dest) = matches.value_of("dest") {
        match dest {
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

fn main() -> anyhow::Result<()> {
    run()?;
    Ok(())
}
