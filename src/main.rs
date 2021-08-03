use std::fs::File;
use std::io::{Write, Read};

mod config;
mod lexer;
mod log;
mod parser;

fn main() {
    // read file to string
    let mut file = File::open("x69-bravo.conf").unwrap();
    let mut source = String::new();
    file.read_to_string(&mut source).unwrap();
    let assembler = config::create_assembler_from_config(&source);
    assembler.logs().iter().for_each(|l| println!("{}", l));
    if let Some(assembler) = assembler.value() {
        let path = std::env::args().nth(1).unwrap();
        let mut file = File::open(path).unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
        
        let code_result = assembler.assemble(&source);
        code_result.logs().iter().for_each(|l| println!("{}", l));
        if let Some(code) = code_result.value() {
            let mut file = File::create("a.out").unwrap();
            file.write_all(&code).unwrap();
        }
    }
}
