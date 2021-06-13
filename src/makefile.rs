use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

// import helper functions from loader module
use crate::loader::{get_var_trimmed, read_bracketed_var};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct FinalRule {
    target: String, // Every rule in the final list only has one target (or target pattern) it provides
    prereqs: Vec<String>,
    recipes: Vec<String>,
}

#[allow(dead_code)]
impl FinalRule {
    // constructor
    pub(crate) fn new(target: String, prereqs: Vec<String>, recipes: Vec<String>) -> Self {
        Self {
            target,
            prereqs,
            recipes,
        }
    }

    // read only member access
    pub(crate) fn target(&self) -> &str {
        &self.target
    }
    pub(crate) fn prereqs(&self) -> &Vec<String> {
        &self.prereqs
    }
    pub(crate) fn recipes(&self) -> &Vec<String> {
        &self.recipes
    }

    // mutable member access
    pub(crate) fn target_mut(&mut self) -> &mut str {
        &mut self.target
    }
    pub(crate) fn prereqs_mut(&mut self) -> &mut Vec<String> {
        &mut self.prereqs
    }
    pub(crate) fn recipes_mut(&mut self) -> &mut Vec<String> {
        &mut self.recipes
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MakeFile {
    var_map: HashMap<String, String>,
    finalised_rules: Vec<FinalRule>,
}

impl MakeFile {
    /// The crate internal constructor for a Makefile
    pub(crate) fn new(var_map: HashMap<String, String>, finalised_rules: Vec<FinalRule>) -> Self {
        MakeFile {
            var_map,
            finalised_rules,
        }
    }

    /// Substitutes variables for their actual value
    fn substitute_var(
        &self,
        it: &mut dyn Iterator<Item = char>,
        target: &str,
        deps: &[String],
    ) -> String {
        match it.next().expect("Unexpected end of variable, quitting!") {
            '@' => target.to_owned(),
            '?' => deps.iter().fold(String::new(), |res, dep| res + " " + dep),
            '<' => deps.iter().fold(String::new(), |res, dep| res + " " + dep),
            '$' => String::from("$"),
            // handle bracketed variables
            '(' => get_var_trimmed(
                &self.var_map,
                read_bracketed_var(it, ")", |it| self.substitute_var(it, target, deps)),
            ),
            '{' => get_var_trimmed(
                &self.var_map,
                read_bracketed_var(it, "}", |it| self.substitute_var(it, target, deps)),
            ),
            x => panic!("${} ???", x),
        }
    }

    /// Performs the build specified by the makefile.
    fn build(&self, target: &FinalRule, silent: bool) -> SystemTime {
        let mut newest_dep: SystemTime = SystemTime::UNIX_EPOCH;
        for prereq in &target.prereqs {
            if let Some(rule) = self.finalised_rules.iter().find(|r| r.target == *prereq) {
                newest_dep = std::cmp::max(self.build(rule, silent), newest_dep);
            } else if !Path::new(prereq).exists() {
                panic!("No rule to build target \"{}\", stopping.", prereq);
            } else {
                newest_dep = std::cmp::max(
                    std::fs::metadata(prereq).unwrap().modified().unwrap(),
                    newest_dep,
                );
            }
        }

        let modified = std::fs::metadata(&target.target)
            .ok()
            .map(|meta| meta.modified().ok())
            .flatten();

        if Path::new(&target.target).exists() && &newest_dep < modified.as_ref().unwrap() {
            return modified.unwrap();
        }

        for recipe in &target.recipes {
            let mut recipe_san = String::new();
            let mut it = recipe.chars().peekable();
            while let Some(c) = it.next() {
                match c {
                    '$' => {
                        recipe_san.push_str(&self.substitute_var(
                            &mut it,
                            &target.target,
                            &target.prereqs,
                        ));
                    }
                    x => {
                        recipe_san.push(x);
                    }
                }
            }

            let mut recipe = recipe_san.trim();
            if recipe.starts_with('@') {
                recipe = recipe[1..].trim();
            }

            if !silent {
                println!("{}", recipe);
            }

            let status = Command::new("sh")
                .arg("-c")
                .arg(recipe)
                .status()
                .expect("Failed to execute process");

            if !status.success() {
                panic!("Program exited with nonzero status, stopping.");
            }
        }
        SystemTime::now()
    }

    /// Builds the default target
    pub fn build_default(&self, silent: bool) {
        let rule = self
            .finalised_rules
            .first()
            .expect("No targets available, quitting!");

        self.build(rule, silent);
    }

    /// Builds a makefile target
    pub fn build_target(&self, target: impl AsRef<str>, silent: bool) {
        let rule = self
            .finalised_rules
            .iter()
            .find(|rule| rule.target == target.as_ref());

        if let Some(rule) = rule {
            self.build(rule, silent);
        } else {
            panic!("No rule to make target {}, quitting!", target.as_ref());
        }
    }
}
