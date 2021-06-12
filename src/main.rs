use clap::{App, Arg};

use std::env;
use std::fs::File;
use std::path::Path;

use lc_make::loader::MakeFileLoader;

fn main() -> std::io::Result<()> {
    let matches = App::new("LC Make")
        .version("0.1.0")
        .author("Ray Redondo <rdrpenguin04@gmail.com>")
        .arg(
            Arg::with_name("dir")
                .short("C")
                .long("directory")
                .takes_value(true)
                .help("Change to <dir> before doing anything"),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .takes_value(true)
                .help("Use <file> as a makefile"),
        )
        .arg(
            Arg::with_name("silent")
                .short("s")
                .help("Don't echo recipes"),
        )
        .arg(Arg::with_name("target"))
        .get_matches();

    // handle -C flag
    if let Some(dir) = matches.value_of("dir") {
        env::set_current_dir(Path::new(dir))?;
    }

    // if the user specified a makefile then use that file
    // otherwise try and find one from a list of defaults
    let file = if let Some(file) = matches.value_of("file") {
        Some(File::open(file))
    } else {
        let defaults = vec!["GNUmakefile", "makefile", "Makefile"];

        defaults.into_iter().map(File::open).find(Result::is_ok)
    };

    // create a new makefile loader
    let mut loader = MakeFileLoader::new();

    // if we have a valid file then load the makefile's contents
    if let Some(Ok(mut file)) = file {
        loader.load(&mut file)?;
    }

    // finalse the loaded makefile
    let makefile = loader.finalise();

    // perform the build
    let silent = matches.is_present("silent");
    if let Some(target) = matches.value_of("target") {
        // don't be silent for debugging purposes
        makefile.build_target(target, silent);
    } else {
        makefile.build_default(silent);
    }

    Ok(())
}
