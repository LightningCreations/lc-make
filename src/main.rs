use argparse::ArgumentParser;

use std::fs::File;
use std::path::PathBuf;

use lc_make::loader::MakeFileLoader;

fn main() -> std::io::Result<()> {
    let mut dir = None::<PathBuf>;
    let mut file = None::<PathBuf>;
    let mut silent = false;
    let mut target = None::<String>;
    let mut ap = ArgumentParser::new();
    ap.refer(&mut dir).add_option(
        &["-C"],
        argparse::StoreOption,
        "Change to the given directory before doing anything else",
    );
    ap.refer(&mut file).add_option(
        &["-f"],
        argparse::StoreOption,
        "Use <file> as the Makefile instead of Makefile",
    );
    ap.refer(&mut target)
        .add_argument("TARGET", argparse::ParseOption, "TARGET to build");
    ap.refer(&mut silent).add_option(
        &["--silent", "-s", "--quiet"],
        argparse::StoreTrue,
        "Prevents make from outputting anything",
    );
    ap.parse_args_or_exit();
    drop(ap);
    if let Some(dir) = dir {
        std::env::set_current_dir(dir)?;
    }
    let file = if let Some(file) = file {
        File::open(file)
    } else {
        let defaults = vec!["GNUmakefile", "makefile", "Makefile"];

        defaults
            .into_iter()
            .map(File::open)
            .find(Result::is_ok)
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot find makefile")
            })
            .and_then(|s| s)
    };

    // create a new makefile loader
    let mut loader = MakeFileLoader::new();

    // if we have a valid file then load the makefile's contents
    if let Ok(mut file) = file {
        loader.load(&mut file)?;
    }

    // finalse the loaded makefile
    let makefile = loader.finalise();

    // perform the build
    if let Some(target) = target {
        // don't be silent for debugging purposes
        makefile.build_target(target, silent);
    } else {
        makefile.build_default(silent);
    }

    Ok(())
}
