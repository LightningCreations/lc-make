use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use crate::makefile::{FinalRule, MakeFile};

// define a consistent message to produce on EOF
const EOF_MESSAGE: &str = "Unexpected EOF";

#[derive(Debug, Clone, Eq, PartialEq)]
enum Var {
    Complex,
    Simple,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum State {
    Left(String), // Processing
    // NOTE: the Var component of RightVariable is never used, is it needed?
    RightVariable(String, Var, String), // Variable name, Is a complex variable, Processing
    RightRule(Vec<String>, String),     // Target names, Processing
    Recipes(Vec<String>, Vec<String>, Vec<String>, String), // Targets, Prereqs, Current list, Processing
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Rule {
    targets: Vec<String>,
    prereqs: Vec<String>,
    recipes: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MakeFileLoader {
    var_map: HashMap<String, String>,
    rule_list: Vec<Rule>,
    include_list: Vec<String>,
}

impl Default for MakeFileLoader {
    fn default() -> Self {
        let mut var_map: HashMap<String, String> = HashMap::new();

        // construct the value for the MAKE variable
        var_map.insert(
            String::from("MAKE"),
            env::current_exe()
                .ok()
                .map(PathBuf::into_os_string)
                .map(|oss| oss.into_string().ok())
                .flatten()
                .unwrap_or_else(|| String::from("make")),
        );

        var_map.insert(String::from("CC"), String::from("cc"));
        var_map.insert(String::from("CXX"), String::from("c++"));

        Self {
            var_map,
            rule_list: Vec::new(),
            include_list: Vec::new(),
        }
    }
}

impl MakeFileLoader {
    /// constructs a new makefile loader with the default parameters
    pub fn new() -> Self {
        Default::default()
    }

    /// loads in all the variables and targets from a given Makefile
    pub fn load(&mut self, file: &mut File) -> std::io::Result<()> {
        // read the content of the makefile
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        // get an iterator over its contents
        let mut it = content.chars().peekable();
        let mut state = State::Left(String::new());

        // when the top is true we don't skip else we skip
        let mut skip_stack = vec![true];
        let mut skip_buf = String::new();

        while let Some(c) = it.next() {
            let skip = !skip_stack.last().expect("missmatched conditonals");
            if skip {
                match c {
                    '\n' => {
                        match skip_buf.trim() {
                            "else" => {
                                if let Some(last) = skip_stack.last_mut() {
                                    // we know we need to stop skipping
                                    // in the else branch if we're here
                                    *last = true
                                } else {
                                    panic!("missmatched conditionals")
                                }
                            }
                            "endif" => {
                                skip_stack.pop().expect("missmatched conditionals");
                            }
                            _ => {}
                        }
                        skip_buf = String::new()
                    }
                    x => {
                        skip_buf.push(x);
                    }
                }
            } else {
                match c {
                    '$' => {
                        let work = match state {
                            State::Left(ref mut work) => work,
                            State::RightRule(_, ref mut work) => work,
                            State::RightVariable(_, _, ref mut work) => work,
                            State::Recipes(_, _, _, ref mut work) => work,
                        };
                        work.push_str(self.substitute_var(&mut it).as_str());
                    }
                    '#' => {
                        while *(it.peek().expect(EOF_MESSAGE)) != '\n' {
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
                                    State::RightVariable(prev, Var::Complex, String::new())
                                }
                                Some('=') => {
                                    it.next();
                                    State::RightVariable(prev, Var::Complex, String::new())
                                }
                                _ => State::RightRule(
                                    prev.split_whitespace().map(str::to_string).collect(),
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
                                state = State::RightVariable(prev, Var::Simple, String::new());
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
                            State::Left(x) if x.trim().is_empty() => State::Left(String::new()),
                            State::Left(x) if x.trim().starts_with("include ") => {
                                let filename = x.trim()[8..].trim();
                                let file = File::open(filename);
                                if let Ok(mut file) = file {
                                    self.load(&mut file)?;
                                } else {
                                    panic!("Couldn't open {}", filename);
                                }
                                State::Left(String::new())
                            }
                            State::Left(x) if x.trim().starts_with("-include ") => {
                                let filename = x.trim()[9..].trim();
                                let file = File::open(filename);
                                if let Ok(mut file) = file {
                                    self.load(&mut file)?;
                                }
                                State::Left(String::new())
                            }
                            State::Left(x) if x.trim().starts_with("ifdef ") => {
                                let rhs = x.trim()[5..].trim();
                                let rhs = if rhs.starts_with("$") {
                                    self.substitute_var(&mut x[1..].chars())
                                } else {
                                    rhs.to_string()
                                };
                                skip_stack.push(self.var_map.contains_key(&rhs));
                                State::Left(String::new())
                            }
                            State::Left(x) if x.trim().starts_with("ifndef ") => {
                                let rhs = x.trim()[6..].trim();
                                let rhs = if rhs.starts_with("$") {
                                    self.substitute_var(&mut x[1..].chars())
                                } else {
                                    rhs.to_string()
                                };
                                skip_stack.push(!self.var_map.contains_key(&rhs));
                                State::Left(String::new())
                            }
                            State::Left(x) if x.trim() == "else" => {
                                if let Some(last) = skip_stack.last_mut() {
                                    // we know we need to start skipping
                                    // in the else branch if we're here
                                    *last = false
                                } else {
                                    panic!("missmatched conditionals")
                                }
                                State::Left(String::new())
                            }
                            State::Left(x) if x.trim() == "endif" => {
                                skip_stack.pop().expect("missmatched conditionals");
                                State::Left(String::new())
                            }
                            State::Left(_) => panic!("Syntax error"),
                            State::RightVariable(name, _, value) => {
                                self.var_map.insert(name.trim().to_owned(), value);
                                State::Left(String::new())
                            }
                            State::RightRule(targets, prereqs) => {
                                while matches!(it.peek(), Some('\n') | Some('#')) {
                                    if let Some('#') = it.next() {
                                        while *(it.peek().expect(EOF_MESSAGE)) != '\n' {
                                            it.next();
                                        }
                                    };
                                }
                                match it.peek() {
                                    Some('\t') => {
                                        it.next(); // Skip \t
                                        State::Recipes(
                                            targets,
                                            prereqs
                                                .split_whitespace()
                                                .map(str::to_string)
                                                .collect(),
                                            Vec::new(),
                                            String::new(),
                                        )
                                    }
                                    _ => {
                                        let prereqs: Vec<String> = prereqs
                                            .split_whitespace()
                                            .map(str::to_string)
                                            .collect();
                                        let recipes = Vec::new();
                                        self.rule_list.push(Rule {
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
                                        while *(it.peek().expect(EOF_MESSAGE)) != '\n' {
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
                                        self.rule_list.push(Rule {
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
                    '\\' => match it.next().expect(EOF_MESSAGE) {
                        '\n' => {
                            let work = match state {
                                State::Left(ref mut work) => work,
                                State::RightVariable(_, _, ref mut work) => work,
                                State::RightRule(_, ref mut work) => work,
                                State::Recipes(_, _, _, ref mut work) => work,
                            };
                            work.push(' ');
                        }
                        x => {
                            let work = match state {
                                State::Left(ref mut work) => work,
                                State::RightVariable(_, _, ref mut work) => work,
                                State::RightRule(_, ref mut work) => work,
                                State::Recipes(_, _, _, ref mut work) => work,
                            };
                            work.push('\\');
                            work.push(x);
                        }
                    },
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

    fn substitute_var(&self, it: &mut dyn Iterator<Item = char>) -> String {
        match it.next().expect(EOF_MESSAGE) {
            // Delay processing until target processing
            '@' => String::from("$@"),
            '?' => String::from("$?"),
            '<' => String::from("$<"),

            '$' => String::from("$"),

            // handle bracketed variables
            '(' => get_var_trimmed(
                &self.var_map,
                read_bracketed_var(it, ")", |it| self.substitute_var(it)),
            ),
            '{' => get_var_trimmed(
                &self.var_map,
                read_bracketed_var(it, "}", |it| self.substitute_var(it)),
            ),

            x => panic!("${} ???", x),
        }
    }

    /// Finalise method consumes the loader object and builds a finalised
    /// version of all the rules, returning the finalised MakeFile object.
    pub fn finalise(self) -> MakeFile {
        let mut final_rule_list: Vec<FinalRule> = Vec::new();
        let mut append_implicit_rules = true;
        let mut inference_rules_warning = false;

        // destructure into variables so we can do move them
        let MakeFileLoader {
            var_map,
            rule_list,
            include_list,
        } = self;

        for rule in rule_list {
            let mut handled = false;
            if rule.targets.len() == 1 {
                if rule.targets[0] == ".POSIX" {
                    handled = true; // We should be POSIX compliant enough; no special flags needed
                } else if rule.targets[0] == ".SUFFIXES" {
                    handled = true;
                    if rule.prereqs.is_empty() {
                        append_implicit_rules = false;
                    } else if rule.prereqs.len() != 1
                        || rule.prereqs[0] != ".hpux_make_needs_suffix_list"
                    {
                        // Workaround for CMake's workaround to a problem we don't have
                        panic!("Unimplemented");
                    }
                } else {
                    let mut it = rule.targets[0].chars();
                    if it.next().expect(EOF_MESSAGE) == '.' {
                        if it.next().expect(EOF_MESSAGE).is_uppercase() {
                            /*
                            println!(
                                "Warning: {} is unimplemented; treating as a normal rule for now",
                                rule.targets[0]
                            );
                            */ // This got annoying fast
                        } else if !inference_rules_warning {
                            handled = true;
                            // println!("Warning: POSIX-style inference rules are unimplemented");
                            inference_rules_warning = true;
                        }
                    }
                }
            }
            if !handled {
                for target in rule.targets {
                    if let Some(existing_rule) =
                        final_rule_list.iter_mut().find(|r| r.target() == target)
                    {
                        existing_rule
                            .prereqs_mut()
                            .append(&mut rule.prereqs.clone());
                        *existing_rule.recipes_mut() = rule.recipes.clone();
                    } else {
                        final_rule_list.push(FinalRule::new(
                            target,
                            rule.prereqs.clone(),
                            rule.recipes.clone(),
                        ));
                    }
                }
            }
        }

        if append_implicit_rules && !inference_rules_warning {
            // println!("Warning: POSIX-style inference rules are unimplemented");
        }

        MakeFile::new(var_map, final_rule_list, include_list)
    }
}

/// gets a variable from a variable map and trims it
pub(crate) fn get_var_trimmed(
    var_map: &HashMap<String, String>,
    variable: impl AsRef<str>,
) -> String {
    let variable = variable.as_ref();
    if variable.starts_with("shell ") {
        let output = Command::new("sh").arg("-c").arg(&variable[6..]).output();
        if let Ok(output) = output {
            String::from_utf8(output.stdout)
                .expect("Command didn't output valid UTF-8")
                .replace("\n", "")
        } else {
            String::new()
        }
    } else {
        var_map
            .get(variable)
            .map(|s| s.as_str().trim().to_owned())
            .unwrap_or_else(String::new)
    }
}

/// function for reading a bracketed variable
/// it is parameterised by a function to substitute
pub(crate) fn read_bracketed_var<S>(
    it: &mut dyn Iterator<Item = char>,
    bracket: impl AsRef<str>,
    sub_var: S,
) -> String
where
    S: Fn(&mut dyn Iterator<Item = char>) -> String,
{
    // abstract reading a variable out to a method
    let mut variable: String = String::new();
    let mut c: String;

    while {
        c = match it.next().expect(EOF_MESSAGE) {
            '#' => panic!("Syntax error"),
            '$' => sub_var(it),
            x => x.to_string(),
        };
        c.as_str()
    } != bracket.as_ref()
    {
        variable.push_str(c.as_str());
    }

    variable
}
