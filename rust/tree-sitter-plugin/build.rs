extern crate cc;

use std::collections::HashSet;
use std::path::PathBuf;

fn main() {
    let root_path: PathBuf = "grammars".into();

    let mut config = cc::Build::new();
    config
        .debug(false)
        .define("UTF8PROC_STATIC", "")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-std=c99");

    // List of misbehaving grammars that we want to skip
    let mut broken_grammars: HashSet<&str> = HashSet::new();
    {
        // Does not compile
        broken_grammars.insert("agda");
        broken_grammars.insert("swift");

        // Requires CXX11 STD
        broken_grammars.insert("bash");
        broken_grammars.insert("cpp");
        broken_grammars.insert("haskell");
        broken_grammars.insert("html");
        broken_grammars.insert("ocaml");
        broken_grammars.insert("php");
        broken_grammars.insert("python");
        broken_grammars.insert("ruby");
    }

    // If we wanted to be fancy, we could maybe use this to also generate code
    // for the extern "C" functions in main
    root_path
        .read_dir()
        .unwrap()
        .map(|it| it.unwrap())
        .filter(|it| !broken_grammars.contains(it.file_name().to_str().unwrap()))
        .for_each(|dir| {
            let src_dir = dir.path().join("src");

            let parser_file = src_dir.join("parser.c");
            let scanner_file = src_dir.join("scanner.c");
            let scanner_file2 = src_dir.join("scanner.cc");

            config.include(&src_dir).file(parser_file);

            if scanner_file.exists() {
                config.file(scanner_file);
            } else if scanner_file2.exists() {
                config.file(scanner_file2);
            }
        });

    config.compile("ts-grammars.a");
}
