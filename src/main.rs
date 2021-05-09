use clap::{App, Arg};

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/*
enum MatchResult {
    Perfect,
    Partial(String), // % in %.c, etc
    Different,
}

fn matches(spec: String, name: String) -> MatchResult {
    MatchResult::Different
}
*/

enum State {
    Left(String),                                           // Processing
    RightVariable(String, bool, String), // Variable name, Is a complex variable, Processing
    RightRule(Vec<String>, String),      // Target names, Processing
    Recipes(Vec<String>, Vec<String>, Vec<String>, String), // Targets, Prereqs, Current list, Processing
}

fn variable_subst(it: &mut dyn Iterator<Item = char>) -> String {
    match it.next() {
        Some('$') => String::from("$"),
        Some('(') => {
            let mut variable: String = String::new();
            let mut c: String;
            while {
                c = match it.next() {
                    Some('#') => panic!("Syntax error"),
                    Some('$') => variable_subst(it),
                    Some(x) => x.to_string(),
                    x => panic!("{:#?}", x),
                };
                &c
            } != ")"
            {
                variable += &c;
            }
            "<value of \"".to_owned() + &variable + "\">"
        }
        Some(x) => panic!("${} ???", x),
        x => panic!("{:#?}", x),
    }
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
        .arg(
            Arg::with_name("file")
                .short("f")
                .takes_value(true)
                .help("Use <file> as a makefile"),
        )
        .get_matches();

    if let Some(dir) = matches.value_of("dir") {
        env::set_current_dir(Path::new(dir))?;
    }

    let file = if let Some(file) = matches.value_of("file") {
        File::open(file)
    } else {
        let mut file = File::open("GNUmakefile");
        if file.is_err() {
            file = File::open("makefile");
        }
        if file.is_err() {
            file = File::open("Makefile");
        }
        file
    };

    if let Ok(mut file) = file {
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let mut it = content.chars().peekable();
        let mut state = State::Left(String::new());
        while let Some(c) = it.next() {
            match c {
                '$' => {
                    let work = match state {
                        State::Left(ref mut work) => work,
                        State::RightRule(_, ref mut work) => work,
                        State::RightVariable(_, _, ref mut work) => work,
                        State::Recipes(_, _, _, ref mut work) => work,
                    };
                    work.push_str(&variable_subst(&mut it));
                }
                '#' => {
                    while *(it.peek().unwrap()) != '\n' {
                        it.next();
                    }
                }
                ':' => match state {
                    State::Left(prev) => {
                        let next = it.peek();
                        state = match next {
                            Some(':') => {
                                it.next();
                                if it.next() != Some('=') {
                                    panic!("Syntax error");
                                }
                                State::RightVariable(prev, true, String::new())
                            }
                            Some('=') => {
                                it.next();
                                State::RightVariable(prev, true, String::new())
                            }
                            _ => State::RightRule(
                                prev.split_whitespace().map(|s| s.to_string()).collect(),
                                String::new(),
                            ),
                        }
                    }
                    State::RightVariable(_, _, ref mut work) => {
                        work.push(':');
                    }
                    State::Recipes(_, _, _, ref mut work) => {
                        work.push(':');
                    }
                    _ => panic!("Syntax error"),
                },
                '=' => {
                    match state {
                        State::Left(prev) => {
                            state = State::RightVariable(prev, false, String::new());
                        }
                        State::RightVariable(_, _, ref mut work) => {
                            work.push('=');
                        }
                        State::Recipes(_, _, _, ref mut work) => {
                            work.push('=');
                        }
                        _ => panic!("Syntax error"),
                    };
                }
                '\n' => {
                    state = match state {
                        State::Left(x) if x.is_empty() => State::Left(String::new()),
                        State::Left(_) => panic!("Syntax error"),
                        State::RightVariable(name, complex, value) => {
                            println!(
                                "Variable \"{}\" with value \"{}\" (is complex = {})",
                                name, value, complex
                            );
                            State::Left(String::new())
                        }
                        State::RightRule(targets, prereqs) => {
                            while matches!(it.peek(), Some('\n') | Some('#')) {
                                if let Some('#') = it.next() {
                                    while *(it.peek().unwrap()) != '\n' {
                                        it.next();
                                    }
                                };
                            }
                            match it.peek() {
                                Some('\t') => {
                                    it.next(); // Skip \t
                                    State::Recipes(
                                        targets,
                                        prereqs.split_whitespace().map(|s| s.to_string()).collect(),
                                        Vec::new(),
                                        String::new(),
                                    )
                                }
                                _ => {
                                    let prereqs: Vec<String> =
                                        prereqs.split_whitespace().map(|s| s.to_string()).collect();
                                    println!(
                                        "Rule for targets {:#?}; prereqs {:#?}",
                                        targets, prereqs
                                    );
                                    State::Left(String::new())
                                }
                            }
                        }
                        State::Recipes(targets, prereqs, mut recipes, work) => {
                            while matches!(it.peek(), Some('\n') | Some('#')) {
                                if let Some('#') = it.next() {
                                    while *(it.peek().unwrap()) != '\n' {
                                        it.next();
                                    }
                                };
                            }
                            recipes.push(work);
                            match it.peek() {
                                Some('\t') => {
                                    it.next(); // Skip \t
                                    State::Recipes(targets, prereqs, Vec::new(), String::new())
                                }
                                _ => {
                                    println!(
                                        "Rule for targets {:#?}; prereqs {:#?}, recipes {:#?}",
                                        targets, prereqs, recipes
                                    );
                                    State::Left(String::new())
                                }
                            }
                        }
                    };
                }
                x => {
                    let work = match state {
                        State::Left(ref mut work) => work,
                        State::RightVariable(_, _, ref mut work) => work,
                        State::RightRule(_, ref mut work) => work,
                        State::Recipes(_, _, _, ref mut work) => work,
                    };
                    work.push(x);
                }
            }
        }
    }
    Ok(())
}
