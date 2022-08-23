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

/// 栈帧接口，不同的体系结构都可以实现这个接口
pub trait Frame: Clone {
    /// Access（访问方式）类型必须实现Clone和Debug
    type Access: Clone + Debug;

    /// 指针的大小
    const WORD_SIZE: i64;

    /// 寄存器列表
    fn registers() -> Vec<Temp>;

    /// 寄存器数量
    fn register_count() -> usize;

    fn temp_map() -> HashMap<Temp, &'static str>;

    fn special_name(temp: Temp) -> Option<&'static str>;

    /// 返回保存帧指针的寄存器
    fn fp() -> Temp;

    /// 返回保存返回值的寄存器
    fn return_value() -> Temp;

    /// 函数调用会生成一个新的栈帧
    /// name是栈帧的名字
    /// formals是函数调用的参数列表
    /// bool表示这个参数是否是逃逸的
    fn new(name: Label, formals: Vec<bool>) -> Self;

    /// 返回栈帧的名字
    fn name(&self) -> Label;

    /// 返回栈帧的参数访问方式的列表
    fn formals(&self) -> &[Self::Access];

    /// 为某个参数指定访问形式
    ///   - 逃逸的：必须放在栈帧中
    ///   - 非逃逸的：可以放在寄存器中
    fn alloc_local(&mut self, escape: bool) -> Self::Access;

    fn exp(&self, access: Self::Access, stack_frame: Exp) -> Exp;

    fn external_call(name: &str, arguments: Vec<Exp>) -> Exp;

    fn proc_entry_exit1(&mut self, statement: Statement) -> Statement;
    fn proc_entry_exit2(&self, instructions: Vec<Instruction>) -> Vec<Instruction>;
    fn proc_entry_exit3(&self, body: Vec<Instruction>) -> Subroutine;
}
