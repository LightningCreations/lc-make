use clap::{Arg, App};

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let matches = App::new("LC Make")
        .version("0.1.0")
        .author("Ray Redondo <rdrpenguin04@gmail.com>")
        .arg(Arg::with_name("dir")
                 .short("C")
                 .long("directory")
                 .takes_value(true)
                 .help("Change to <dir> before doing anything"))
        .get_matches();

    if let Some(dir) = matches.value_of("dir") {
        env::set_current_dir(Path::new(dir))?;
    }

    let mut file = File::open("GNUmakefile");
    if file.is_err() {
        file = File::open("makefile");
    }
    if file.is_err() {
        file = File::open("Makefile");
    }
    let mut file = file?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    println!("{}", content);
    Ok(())
}
