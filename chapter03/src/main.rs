mod ast;
mod symbol;
mod error;
mod terminal;
mod position;
mod token;
mod lexer;
mod parser;

use std::env::args;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

use error::Error;
use lexer::Lexer;
use symbol::{Strings, Symbols};
use parser::Parser;
use terminal::Terminal;

fn main() {
    let strings = Rc::new(Strings::new());
    let mut symbols = Symbols::new(Rc::clone(&strings));
    if let Err(error) = drive(&mut symbols) {
        let terminal = Terminal::new();
        if let Err(error) = error.show(&symbols, &terminal) {
            eprintln!("Error printing errors: {}", error);
        }
    }
}

fn drive(symbols: &mut Symbols<()>) -> Result<(), Error> {
    let mut args = args();
    args.next();
    if let Some(filename) = args.next() {
        let file = BufReader::new(File::open(&filename)?);
        let file_symbol = symbols.symbol(&filename);
        let lexer = Lexer::new(file, file_symbol);
        let mut parser = Parser::new(lexer, symbols);
        let ast = parser.parse()?;
        println!("{:?}", ast);
    }
    Ok(())
}