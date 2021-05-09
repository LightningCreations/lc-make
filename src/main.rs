use clap::{App, Arg};

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

enum MatchResult {
    Perfect,
    Partial(String), // % in %.c, etc
    Different,
}

fn matches(spec: String, name: String) -> MatchResult {
    return MatchResult::Different;
}

enum State {
    Left(String),                                           // Processing
    RightVariable(String, String),                          // Variable name, Processing
    RightRule(Vec<String>, String),                         // Target names, Processing
    Recipes(Vec<String>, Vec<String>, Vec<String>, String), // Targets, Prereqs, Current list, Processing
}

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
    if let Ok(mut file) = file {
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let mut it = content.chars().peekable();
        let mut state = State::Left(String::new());
        while let Some(c) = it.next() {
            match c {
                ':' => {
                    state = match state {
                        State::Left(prev) => {
                            let next = it.peek();
                            match next {
                                Some(':') => {
                                    it.next();
                                    if it.next() != Some('=') {
                                        panic!("Syntax error");
                                    }
                                    State::RightVariable(prev, String::new())
                                }
                                Some('=') => {
                                    it.next();
                                    State::RightVariable(prev, String::new())
                                }
                                _ => State::RightRule(
                                    prev.split_whitespace().map(|s| s.to_string()).collect(),
                                    String::new(),
                                ),
                            }
                        }
                        _ => panic!("Syntax error"),
                    }
                }
                '\n' => {
                    state = match state {
                        State::Left(x) if x.is_empty() => State::Left(String::new()),
                        _ => unreachable!(),
                    };
                }
                _ => {
                    panic!("Unimpled");
                }
            }
        }
    }
    Ok(())
}
