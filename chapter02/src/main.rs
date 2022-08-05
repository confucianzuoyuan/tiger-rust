mod error;
mod terminal;
mod position;
mod token;
mod lexer;

use std::env::args;
use std::fs::File;
use std::io::BufReader;

use error::Error;
use lexer::Lexer;
use terminal::Terminal;

fn main() {
    if let Err(error) = drive() {
        let terminal = Terminal::new();
        if let Err(error) = error.show(&terminal) {
            eprintln!("Error printing errors: {}", error);
        }
    }
}

fn drive() -> Result<(), Error> {
    let mut args = args();
    args.next();
    if let Some(filename) = args.next() {
        let file = BufReader::new(File::open(&filename)?);
        let mut lexer = Lexer::new(file, &filename);
        loop {
            let token = lexer.token();
            match token {
                Ok(token) => println!("{:?}", token),
                Err(Error::Eof) => break,
                Err(error) => return Err(error),
            }
        }
    }
    Ok(())
}