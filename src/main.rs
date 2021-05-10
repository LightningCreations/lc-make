use clap::{App, Arg};

use std::collections::HashMap;
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

#[derive(Debug)]
struct FinalRule {
    target: String, // Ever rule in the final list only has one target (or target pattern) it provides
    prereqs: Vec<String>,
    recipes: Vec<String>,
}

struct Rule {
    targets: Vec<String>,
    prereqs: Vec<String>,
    recipes: Vec<String>,
}

fn variable_subst(
    it: &mut dyn Iterator<Item = char>,
    var_map: &mut HashMap<String, String>,
) -> String {
    match it.next() {
        Some('$') => String::from("$"),
        Some('(') => {
            let mut variable: String = String::new();
            let mut c: String;
            while {
                c = match it.next() {
                    Some('#') => panic!("Syntax error"),
                    Some('$') => variable_subst(it, var_map),
                    Some(x) => x.to_string(),
                    x => panic!("{:#?}", x),
                };
                &c
            } != ")"
            {
                variable += &c;
            }
            var_map.get(&variable).unwrap_or(&String::new()).clone()
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

    let mut var_map: HashMap<String, String> = HashMap::new();

    var_map.insert(
        String::from("MAKE"),
        env::current_exe()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap_or(String::from("make")),
    );

    let mut rule_list: Vec<Rule> = Vec::new();

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
                    work.push_str(&variable_subst(&mut it, &mut var_map));
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
                        State::RightVariable(name, _, value) => {
                            var_map.insert(name.trim().to_owned(), value);
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
                                    let recipes = Vec::new();
                                    rule_list.push(Rule {
                                        targets,
                                        prereqs,
                                        recipes,
                                    });
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
                                    State::Recipes(targets, prereqs, recipes, String::new())
                                }
                                _ => {
                                    rule_list.push(Rule {
                                        targets,
                                        prereqs,
                                        recipes,
                                    });
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

    let mut final_rule_list: Vec<FinalRule> = Vec::new();
    let mut append_implicit_rules = true;
    let mut inference_rules_warning = false;

    for rule in rule_list {
        let mut handled = false;
        if rule.targets.len() == 1 {
            if rule.targets[0] == ".SUFFIXES" {
                handled = true;
                if rule.prereqs.len() == 0 {
                    append_implicit_rules = false;
                } else if rule.prereqs.len() != 1
                    || rule.prereqs[0] != ".hpux_make_needs_suffix_list"
                {
                    // Workaround for CMake's workaround to a problem we don't have
                    panic!("Unimplemented");
                }
            } else {
                let mut it = rule.targets[0].chars();
                if it.next().unwrap() == '.' {
                    if it.next().unwrap().is_uppercase() {
                        println!(
                            "Warning: {} is unimplemented; treating as a normal rule for now",
                            rule.targets[0]
                        );
                    } else if !inference_rules_warning {
                        println!("Warning: POSIX-style inference rules are unimplemented");
                        inference_rules_warning = true;
                    }
                }
            }
        }
        if !handled {
            for target in rule.targets {
                let mut handled = false;
                for existing_rule in &mut final_rule_list {
                    if existing_rule.target == target {
                        existing_rule.prereqs.append(&mut rule.prereqs.clone());
                        existing_rule.recipes = rule.recipes.clone();
                        handled = true;
                        break;
                    }
                }
                if !handled {
                    final_rule_list.push(FinalRule {
                        target,
                        prereqs: rule.prereqs.clone(),
                        recipes: rule.recipes.clone(),
                    });
                }
            }
        }
    }
    if append_implicit_rules && !inference_rules_warning {
        println!("Warning: POSIX-style inference rules are unimplemented");
    }

    println!("{:#?}", final_rule_list);

    Ok(())
}
