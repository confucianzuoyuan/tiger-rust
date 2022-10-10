use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use asm::{Instruction, Subroutine};
use ir::{Exp, Statement};
use temp::{Label, Temp};

pub mod x86_64;

pub enum Fragment<F: Frame> {
    Function {
        body: Statement,
        frame: Rc<RefCell<F>>,
    },
    Str(Label, String),
}

pub trait Frame: Clone {
    type Access: Clone + Debug;

    const WORD_SIZE: i64;

    fn registers() -> Vec<Temp>;
    fn register_count() -> usize;
    fn temp_map() -> HashMap<Temp, &'static str>;
    fn special_name(temp: Temp) -> Option<&'static str>;

    fn fp() -> Temp;
    fn return_value() -> Temp;

    fn new(name: Label, formals: Vec<bool>) -> Self;

    fn name(&self) -> Label;

    fn formals(&self) -> &[Self::Access];

    fn alloc_local(&mut self, escape: bool) -> Self::Access;

    fn exp(&self, access: Self::Access, stack_frame: Exp) -> Exp;

    fn external_call(name: &str, arguments: Vec<Exp>) -> Exp;

    fn proc_entry_exit1(&mut self, statement: Statement) -> Statement;
    fn proc_entry_exit2(&self, instructions: Vec<Instruction>) -> Vec<Instruction>;
    fn proc_entry_exit3(&self, body: Vec<Instruction>) -> Subroutine;
}
