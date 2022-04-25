use std::cmp::Ordering;
use std::fs;
use std::path::Path;
use std::fmt;
use std::io::{stdin, Write, stdout};

use crate::global;

pub fn select_largest<T: std::cmp::PartialOrd>(a: T, b: T) -> T {
    match a.partial_cmp(&b) {
        None => a,
        Some(c) => {
            match c {
                Ordering::Less => b,
                _ => a
            }
        }
    }
}

pub fn get_non_repeating_filename(full_path: &str) -> Result<String, String> {
    let (mut folder_path, full_filename) = get_last(full_path, '/'); // split input into folder path and name

    if folder_path == "" { // if full_path is just a filename, search current dir
        folder_path = "."
    }

    let dir = match fs::read_dir(Path::new(folder_path)) { // get items in directory
        Ok(rd) => rd,
        Err(_) => return Err(format!("could not read dir {}", folder_path))
    };

    let mut files: Vec<String> = Vec::new();

    let (filename, extension) = get_last(full_filename, '.'); // split name from extension

    for e in dir {
        let entry_name = e.unwrap().file_name().into_string().unwrap(); // hope it doesnt error here
        if string_ends_with(&entry_name, extension) { // if extensions are the same, add it to the list to be checked
            files.push(entry_name)
        }
    }

    let mut name_exists = false;
    for f in &files {
        if f == &full_filename {
            name_exists = true;
            break
        }
    }
    if !name_exists { // name does not exist on first pass - input path/name is free
        return Ok(full_path.to_string())
    }

    let mut iter = 1;
    let mut check_string;
    loop {
        check_string = format!("{}_{}.{}", filename, iter, extension); // eg file_1.ext
        let mut name_exists = false;
        for f in &files {
            if f == &check_string {
                name_exists = true;
                break // stop checking on hit
            }
        }
        if !name_exists { // if name is free, break
            break
        }
        iter += 1 // increase number and check again
    }

    Ok(format!("{}/{}", folder_path, check_string))
}

pub fn get_last(s: &str, delim: char) -> (&str, &str) { // (name, ext)
    //dbg!(s);
    //dbg!(delim);
    let split: Vec<&str> = s.split(delim).collect();
    //dbg!(&split);
    let last = split[split.len() - 1];

    let main = if split.len() != 1 {
        &s[..s.len() - (last.len() + 1)] // slice the input string instead of gluing vec back together
    }
    else {
        ""
    }; // if the input string doesnt split down, return empty string as main

    (main, last)
}

fn string_ends_with(s: &str, e: &str) -> bool { // string, end
    if e.len() > s.len() {
        return false
    }
    &s[s.len() - e.len()..] == e // i got nothin on this one
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

    String::from(&ret[..ret.len()-2]) // remove trailing comma
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

pub fn confirm(prompt: &str) -> bool {
    loop {
        print!("{} ", prompt);
        let _ = stdout().flush();
        let mut buf = String::new();
        let _ = stdin().read_line(&mut buf);
        //println!("{}", buf);
        match buf.trim().to_lowercase().as_str() {
            "yes"|"y" => {
                return true
            }
            "no"|"n" => {
                return false
            }
            _ => {
                println!("please select y/n")
            }
        }

    }
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
    #[test]
    fn test_split_ext() {
        assert_eq!(get_last("file.e", '.'), ("file", "e"));
        assert_eq!(get_last("file.beans.e", '.'), ("file.beans", "e"));
    }
    #[test]
    fn combined_test() {
        let (_, last) = get_last("beans.file", '.');
        assert!(string_ends_with("beans.file", last))
    }
}
