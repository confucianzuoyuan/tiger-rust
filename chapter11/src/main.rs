#![feature(box_patterns)]

mod asm;
mod asm_gen;
mod ast;
mod canon;
mod color;
mod env;
mod error;
mod escape;
mod flow;
mod frame;
mod gen;
mod graph;
mod ir;
mod lexer;
mod liveness;
mod parser;
mod position;
mod reg_alloc;
mod semant;
mod symbol;
mod temp;
mod terminal;
mod token;
mod types;

use asm_gen::Gen;
use reg_alloc::alloc;
use std::env::args;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

use canon::{basic_blocks, linearize, trace_schedule};
use env::Env;
use error::Error;
use escape::find_escapes;
use frame::x86_64::X86_64;
use frame::{Fragment, Frame};
use lexer::Lexer;
use parser::Parser;
use semant::SemanticAnalyzer;
use symbol::{Strings, Symbols};
use terminal::Terminal;

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
        let escape_env = find_escapes(&ast, Rc::clone(&strings));
        let mut env = Env::<X86_64>::new(&strings, escape_env);
        let semantic_analyzer = SemanticAnalyzer::new(&mut env, Rc::clone(&strings));
        let fragments = semantic_analyzer.analyze(main_symbol, ast)?;

        for fragment in fragments {
            match fragment {
                Fragment::Function { body, frame } => {
                    let mut frame = frame.borrow_mut();

                    let statements = linearize(body);
                    let (basic_blocks, done_label) = basic_blocks(statements);
                    let statements = trace_schedule(basic_blocks, done_label);

                    let mut generator = Gen::new();
                    for statement in statements {
                        generator.munch_statement(statement);
                    }
                    let instructions = generator.get_result();
                    let instructions = frame.proc_entry_exit2(instructions);
                    let instructions = alloc::<X86_64>(instructions, &mut *frame);

                    for instruction in instructions {
                        println!("{}", instruction.to_string::<X86_64>());
                    }
                }
                Fragment::Str(_, _) => (),
            }
        }
    }
    Ok(())
}
