use std::fs::File;
use std::io::{Write, Read};

// mod config;
// mod parser;

mod lexer;
mod log;
mod new_parser;

fn main() {
    // read file to string
    let mut file = File::open("x69-bravo.conf").unwrap();
    let mut config_source = String::new();
    file.read_to_string(&mut config_source).unwrap();
    
    
    
    // let (assembler, logs) = config::create_assembler_from_config(&source).unwrap();
    // logs.iter().for_each(|l| println!("{}", l));
    // if let Some(assembler) = assembler {
    //     let path = std::env::args().nth(1).unwrap();
    //     let mut file = File::open(path).unwrap();
    //     let mut source = String::new();
    //     file.read_to_string(&mut source).unwrap();
        
    //     let (code, logs) = assembler.assemble(&source).unwrap();
    //     logs.iter().for_each(|l| println!("{}", l));
    //     if let Some(code) = code {
    //         let mut file = File::create("a.out").unwrap();
    //         file.write_all(&code).unwrap();
    //     }
    // }
}
