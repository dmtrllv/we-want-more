use std::{collections::HashMap, env::args};

pub struct Args {
    flags: Vec<String>,
    args: HashMap<String, String>,
    values: Vec<String>,
}

impl Args {
    pub fn new() -> Self {
        let mut flags = vec![];
        let mut args_map = HashMap::new();
        let mut values = vec![];

        let mut iter = args();

        while let Some(arg) = iter.next() {
            if let Some(arg) = arg.strip_prefix("--") {
                flags.push(arg.to_string());
            } else if let Some(arg) = arg.strip_prefix("-") {
                match iter.next() {
                    Some(val) if !val.starts_with("-") => {
                        args_map.insert(arg.to_string(), val);
                    }
                    _ => println!("missing argument value for --{arg}! (skipping)"),
                }
            } else {
                values.push(arg);
            }
        }

        Self {
            flags,
            args: args_map,
            values,
        }
    }

    pub fn get_arg(&self, name: &str) -> Option<&String> {
        self.args.get(name)
    }

    pub fn has_command(&self, name: &str) -> bool {
        self.values.contains(&name.to_string())
    }

    pub fn has_flag(&self, name: &str) -> bool {
        self.flags.contains(&name.to_string())
    }
}
