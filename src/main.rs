mod errors;
mod pixelmatch;

use clap::{App, Arg};
use errors::{ErrorKind, PixelMatchError};
use std::io::{self, Write};

pub type Result<T> = ::std::result::Result<T, PixelMatchError>;

fn run() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("pixelmatch")
        .arg(
            Arg::with_name("threshold")
                .help("threshold")
                .long("threshold")
                .default_value("0.1")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("aa")
                .help("is include antialiased")
                .long("include-antialiased"),
        )
        .arg(
            Arg::with_name("dest")
                .help("destination path")
                .short("d")
                .long("dest")
                .default_value("-")
                .takes_value(true),
        )
        .arg(Arg::with_name("before").index(1).required(true))
        .arg(Arg::with_name("after").index(2).required(true))
        .get_matches();

    let threshold: f64 = matches.value_of("threshold").unwrap().parse()?;
    let is_include_aa = matches.is_present("aa");

    let before = matches.value_of("before").unwrap();
    let after = matches.value_of("after").unwrap();
    let img1 = image::open(before)?;
    let img2 = image::open(after)?;

    let mut out = image::ImageBuffer::new(256, 256);

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
        };
    };
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        let message = e.to_string();

        io::stderr()
            .write_all(&format!("caused: {}", message).into_bytes())
            .unwrap();

        match e.kind() {
            ErrorKind::IOError => {
                std::process::exit(1);
            }
            ErrorKind::InvalidImageFile => {
                std::process::exit(1);
            }
            ErrorKind::ParseFloatError => {
                std::process::exit(2);
            }
            ErrorKind::SizeUnmatch => {
                std::process::exit(2);
            }
        };
    };
    std::process::exit(0);
}
