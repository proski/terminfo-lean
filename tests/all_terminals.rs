use std::fs;

use terminfo_lean::{
    locate::{locate, search_directories},
    parse::parse,
};

#[test]
fn test_all_terminals() {
    let dirs = search_directories();
    for dir in dirs {
        let Ok(dir) = fs::read_dir(&dir) else {
            continue;
        };
        for leaf in dir {
            let leaf = leaf.unwrap().path();
            let Ok(leaf) = fs::read_dir(&leaf) else {
                continue;
            };
            for term in leaf {
                let term_name = term.unwrap().file_name();
                let terminfo_path = locate(&term_name).unwrap();
                let terminfo_buffer = fs::read(terminfo_path).unwrap();
                let terminfo = parse(&terminfo_buffer).unwrap();
                println!("terminal: {term_name:?}");
                for key in terminfo.booleans {
                    println!("\t{key},");
                }
                for (key, value) in terminfo.numbers {
                    println!("\t{key}#{value},");
                }
                for (key, value) in terminfo.strings {
                    println!("\t{key}={:?},", String::from_utf8_lossy(value));
                }
            }
        }
    }
}
