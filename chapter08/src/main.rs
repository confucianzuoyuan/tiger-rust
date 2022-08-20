mod ast;
mod canon;
mod env;
mod error;
mod escape;
mod frame;
mod gen;
mod ir;
mod lexer;
mod parser;
mod position;
mod semant;
mod symbol;
mod temp;
mod terminal;
mod token;
mod types;

use std::env::args;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

use env::Env;
use error::Error;
use escape::find_escapes;
use frame::x86_64::X86_64;
use frame::Fragment;
use lexer::Lexer;
use parser::Parser;
use semant::SemanticAnalyzer;
use symbol::{Strings, Symbols};
use terminal::Terminal;
use canon::{basic_blocks, linearize, trace_schedule};

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
        let main_symbol = symbols.symbol("main");
        let mut parser = Parser::new(lexer, symbols);
        let ast = parser.parse()?;
        println!("\n========= ast =============\n");
        parser.pp_expr(&ast, 0);
        println!("\n========= ast =============\n");
        let escape_env = find_escapes(&ast, Rc::clone(&strings));
        let mut env = Env::<X86_64>::new(&strings, escape_env);
        let semantic_analyzer = SemanticAnalyzer::new(&mut env, Rc::clone(&strings));
        let fragments = semantic_analyzer.analyze(main_symbol, ast)?;
        println!("\n========== ir ============\n");
        for fragment in &fragments {
            match fragment {
                Fragment::Function { body, .. } => println!("{:#?}", body),
                Fragment::Str(_, string) => println!("String {}", string),
            }
        }
        println!("\n========== ir ============\n\n\n");
        println!("============= ir after canon ========\n");
        for fragment in fragments {
            match fragment {
                Fragment::Function { body, .. } => {
                    let statements = linearize(body);
                    let (basic_blocks, done_label) = basic_blocks(statements);
                    let statements = trace_schedule(basic_blocks, done_label);
                    println!("{:#?}", statements);
                },
                Fragment::Str(_, _) => (),
            }
        }
        println!("\n============= ir after canon ========\n");
    }
    Ok(())
}
