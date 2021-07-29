use std::fs::File;
use std::io::Read;

mod config;
mod lexer;
mod log;

fn main() {
    // read file to string
    let mut file = File::open("x69-bravo.conf").unwrap();
    let mut source = String::new();
    file.read_to_string(&mut source).unwrap();
    let (assembler, logs) = config::parse(&source);
    logs.iter().for_each(|l| println!("{}", l));
    if let Some(assembler) = assembler {
        assembler.instructions.iter().for_each(|(k, v)| println!("{:?}, {:#?}", k, v));
        assembler.instructions.iter().for_each(|(_, v)| println!("{:?}", v.syntaxes));
    }
}
