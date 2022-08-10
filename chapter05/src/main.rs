mod ast;
mod symbol;
mod error;
mod ir;
mod terminal;
mod position;
mod token;
mod lexer;
mod parser;
mod env;
mod types;
mod semant;

use std::env::args;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

use env::Env;
use error::Error;
use lexer::Lexer;
use symbol::{Strings, Symbols};
use parser::Parser;
use terminal::Terminal;

use crate::semant::SemanticAnalyzer;

fn main() {
    let strings = Rc::new(Strings::new());
    let mut symbols = Symbols::new(Rc::clone(&strings));
    if let Err(error) = drive(strings, &mut symbols) {
        let terminal = Terminal::new();
        if let Err(error) = error.show(&symbols, &terminal) {
            eprintln!("Error printing errors: {}", error);
        }
    }
}

fn drive(strings: Rc<Strings>, symbols: &mut Symbols<()>) -> Result<(), Error> {
    let mut args = args();
    args.next();
    if let Some(filename) = args.next() {
        let file = BufReader::new(File::open(&filename)?);
        let file_symbol = symbols.symbol(&filename);
        let lexer = Lexer::new(file, file_symbol);
        let mut parser = Parser::new(lexer, symbols);
        let ast = parser.parse()?;
        parser.pp_expr(&ast, 0);
        println!("");
        let mut env = Env::new(&strings);
        let semantic_analyzer = SemanticAnalyzer::new(&mut env);
        let ir = semantic_analyzer.analyze(ast)?;
        println!("{:?}", ir);
    }
    Ok(())
}