use std::cmp::Ordering;
use std::fs;
use std::path::Path;
use std::fmt;

use super::global;

pub fn select_largest<T: std::cmp::PartialOrd>(a: T, b: T) -> T { // generic cuz im feeling like a clever fucker today
    match a.partial_cmp(&b) {
        None => a,
        Some(c) => {
            match c {
                Ordering::Less => b,
                _ => a // faster than matching greater and equal separately. rustc probably knows the optimisation but oh well
            }
        }
    }
}

pub fn get_non_repeating_filename(folder_path: &str, filename: &str, extension: &str) -> Result<String, String> { // does NOT want dots
    let dir = match fs::read_dir(Path::new(folder_path)) {
        Ok(rd) => rd,
        Err(_) => return Err(format!("could not read dir {}", folder_path))
    };

    let mut files: Vec<String> = Vec::new();

    for e in dir {
        let entry_name = e.unwrap().file_name().into_string().unwrap();
        if string_ends_with(&entry_name, extension) {
            files.push(entry_name)
        }
    }

    let mut name_exists = false;
    let check_string = format!("{}.{}", filename, extension);
    for f in &files {
        if f == &check_string {
            name_exists = true;
            break
        }
    }
    if !name_exists {
        return Ok(format!("{}.{}", filename, extension))
    }

    let mut iter = 1;
    let mut check_string;
    loop {
        check_string = format!("{}_{}.{}", filename, iter, extension);
        let mut name_exists = false;
        for f in &files {
            if f == &check_string {
                name_exists = true;
                break
            }
        }
        if !name_exists {
            break
        }
        iter += 1
    }

    Ok(check_string)
}

fn string_ends_with(s: &str, e: &str) -> bool { // string, end
    if e.len() > s.len() {
        return false
    }
    &s[s.len() - e.len()..s.len()] == e
}

#[derive(Default)]
pub struct ProgramOptions {
    pub global_data: global::GwGlobalData,
    pub verbosity: i32,
    pub logging: bool
}

impl ProgramOptions {
    #[allow(dead_code)]
    pub fn new(global_data: global::GwGlobalData) -> Self {
        ProgramOptions {
            global_data,
            ..ProgramOptions::default()
        }
    }
}

pub fn fmt_vec<T: fmt::Display>(v: &Vec<T>) -> String {
    let mut ret = String::new();
    for i in v {
        ret.push_str(&format!("{}, ", i))
    }

    String::from(&ret[..ret.len()-2])
}

pub fn fmt_option<T: fmt::Display>(o: &Option<T>) -> String {
    match o {
        Some(v) => format!("{}", v),
        None => String::from("none")
    }
}

pub fn fmt_vec_with_tabs<T: fmt::Display>(v: &Vec<T>, number_of_tabs: usize) -> String {
    let mut ret = String::new();
    for i in v {
        for _ in 0..number_of_tabs {
            ret.push('\t')
        }
        ret.push_str(&format!("{}", i));
        ret.push('\n')
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_string_end() {
        assert!(string_ends_with("beans.file", "file"));
        assert!(!string_ends_with("beans.file", "wrong"));
        assert!(!string_ends_with("beans", "looooong"));
    }
}