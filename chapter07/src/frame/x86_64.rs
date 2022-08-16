use std::collections::HashMap;
use std::sync::Once;

use ir::BinOp::Plus;
use ir::Exp:: {
    self,
    BinOp,
    Call,
    Const,
    Mem,
    Name,
};

use super::Frame;
use temp::{Label, Temp};

use self::Access::{InFrame, InReg};

/// x64体系结构中，指针的大小是8个字节
const POINTER_SIZE: i64 = 8;

#[derive(Clone)]
pub struct X86_64 {
    formals: Vec<Access>,
    name: Label,
    pointer: i64,
}

impl PartialEq for X86_64 {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// 对变量的访问形式，分两种情况
///   - 逃逸变量必须存储在栈帧中，InFrame(offset)，offset是相对于fp的偏移量
///   - 非逃逸变量存储在寄存器中，InReg(Temp)，Temp是某个临时变量（虚拟寄存器）
#[derive(Clone, Debug)]
pub enum Access {
    InFrame(i64),
    InReg(Temp),
}

/// "rbp"是保存帧指针的寄存器
static mut RBP : Option<Temp> = None;
/// "rsp"是保存栈指针的寄存器
static mut RSP : Option<Temp> = None;
/// "rax"是保存返回值的寄存器
static mut RAX : Option<Temp> = None;
static mut RBX : Option<Temp> = None;
static mut RCX : Option<Temp> = None;
static mut RDX : Option<Temp> = None;
static mut RSI : Option<Temp> = None;
static mut RDI : Option<Temp> = None;
static mut R8  : Option<Temp> = None;
static mut R9  : Option<Temp> = None;
static mut R10 : Option<Temp> = None;
static mut R11 : Option<Temp> = None;
static mut R12 : Option<Temp> = None;
static mut R13 : Option<Temp> = None;
static mut R14 : Option<Temp> = None;
static mut R15 : Option<Temp> = None;
static ONCE: Once = Once::new();

fn initialize() {
    unsafe {
        RBP = Some(Temp::new());
        RSP = Some(Temp::new());
        RAX = Some(Temp::new());
        RBX = Some(Temp::new());
        RCX = Some(Temp::new());
        RDX = Some(Temp::new());
        RDI = Some(Temp::new());
        RSI = Some(Temp::new());
        R8  = Some(Temp::new());
        R9  = Some(Temp::new());
        R10 = Some(Temp::new());
        R11 = Some(Temp::new());
        R12 = Some(Temp::new());
        R13 = Some(Temp::new());
        R14 = Some(Temp::new());
        R15 = Some(Temp::new());
    }
}

impl X86_64 {
    /// x86_64的ABI要求使用以下六个寄存器来传递函数调用的参数
    pub fn arg_registers() -> Vec<Temp> {
        vec![
            Self::rdi(),
            Self::rsi(),
            Self::rdx(),
            Self::rcx(),
            Self::r8(),
            Self::r9(),
        ]
    }

    /// 被调用者保存的寄存器
    pub fn callee_saved_registers() -> Vec<Temp> {
        vec![
            Self::rbx(),
            Self::rbp(),
            Self::r12(),
            Self::r13(),
            Self::r14(),
            Self::r15(),
        ]
    }

    /// 两个特殊寄存器
    ///   - rax：保存返回值的寄存器
    ///   - rsp：保存栈指针的寄存器
    fn special_registers() -> Vec<Temp> {
        vec![
            Self::rax(),
            Self::rsp(),
        ]
    }

    /// 调用者保存的寄存器
    fn caller_saved_registers() -> Vec<Temp> {
        vec![
            Self::r10(),
            Self::r11(),
        ]
    }

    pub fn rsp() -> Temp {
        ONCE.call_once(initialize);
        unsafe { RSP.expect("temp") }
    }

    fn rbp() -> Temp {
        ONCE.call_once(initialize);
        unsafe { RBP.expect("temp") }
    }

    fn rdi() -> Temp {
        ONCE.call_once(initialize);
        unsafe { RDI.expect("temp") }
    }

    fn rsi() -> Temp {
        ONCE.call_once(initialize);
        unsafe { RSI.expect("temp") }
    }

    pub fn rax() -> Temp {
        ONCE.call_once(initialize);
        unsafe { RAX.expect("temp") }
    }

    fn rbx() -> Temp {
        ONCE.call_once(initialize);
        unsafe { RBX.expect("temp") }
    }

    fn rcx() -> Temp {
        ONCE.call_once(initialize);
        unsafe { RCX.expect("temp") }
    }

    pub fn rdx() -> Temp {
        ONCE.call_once(initialize);
        unsafe { RDX.expect("temp") }
    }

    fn r8() -> Temp {
        ONCE.call_once(initialize);
        unsafe { R8.expect("temp") }
    }

    fn r9() -> Temp {
        ONCE.call_once(initialize);
        unsafe { R9.expect("temp") }
    }

    fn r10() -> Temp {
        ONCE.call_once(initialize);
        unsafe { R10.expect("temp") }
    }

    fn r11() -> Temp {
        ONCE.call_once(initialize);
        unsafe { R11.expect("temp") }
    }

    fn r12() -> Temp {
        ONCE.call_once(initialize);
        unsafe { R12.expect("temp") }
    }

    fn r13() -> Temp {
        ONCE.call_once(initialize);
        unsafe { R13.expect("temp") }
    }

    fn r14() -> Temp {
        ONCE.call_once(initialize);
        unsafe { R14.expect("temp") }
    }

    fn r15() -> Temp {
        ONCE.call_once(initialize);
        unsafe { R15.expect("temp") }
    }
}

/// X86_64的栈帧结构
impl Frame for X86_64 {
    type Access = Access;

    const WORD_SIZE : i64 = 8;

    fn registers() -> Vec<Temp> {
        let mut registers = Self::arg_registers();
        registers.push(Self::return_value());
        registers.extend(Self::callee_saved_registers());
        registers.extend(Self::special_registers());
        registers.extend(Self::caller_saved_registers());
        registers
    }

    fn register_count() -> usize {
        Self::registers().len() - [Self::rsp(), Self::rbp()].len()
    }

    fn temp_map() -> HashMap<Temp, &'static str> {
        let mut map = HashMap::new();
        map.insert(Self::rbp(), "rbp");
        map.insert(Self::rsp(), "rsp");
        map.insert(Self::return_value(), "rax");
        map.insert(Self::rbx(), "rbx");
        map.insert(Self::rdi(), "rdi");
        map.insert(Self::rsi(), "rsi");
        map.insert(Self::rdx(), "rdx");
        map.insert(Self::rcx(), "rcx");
        map.insert(Self::r8(), "r8");
        map.insert(Self::r9(), "r9");
        map.insert(Self::r10(), "r10");
        map.insert(Self::r11(), "r11");
        map.insert(Self::r12(), "r12");
        map.insert(Self::r13(), "r13");
        map.insert(Self::r14(), "r14");
        map.insert(Self::r15(), "r15");
        map
    }

    fn special_name(temp : Temp) -> Option<&'static str> {
        Self::temp_map().get(&temp).map(|&str| str)
    }

    fn fp() -> Temp {
        Self::rbp()
    }

    fn return_value() -> Temp {
        Self::rax()
    }

    /// 实例化时，针对一个变量是否为逃逸的
    /// 来为变量指定访问方式
    fn new(name: Label, formals: Vec<bool>) -> Self {
        let mut frame = X86_64 {
            formals: vec![],
            name,
            pointer: 0,
        };
        let formals = formals
            .iter()
            .map(|&escape| frame.alloc_local(escape))
            .collect();
        frame.formals = formals;
        frame
    }

    /// 返回栈帧的名字
    fn name(&self) -> Label {
        self.name.clone()
    }

    /// 返回栈帧的参数列表
    fn formals(&self) -> &[Self::Access] {
        &self.formals
    }

    /// 为变量指定访问方式
    fn alloc_local(&mut self, escape: bool) -> Self::Access {
        if escape {
            self.pointer -= POINTER_SIZE;
            InFrame(self.pointer)
        } else {
            InReg(Temp::new())
        }
    }

    fn exp(&self, access : Self::Access, stack_frame : Exp) -> Exp {
        match access {
            InFrame(pos) => {
                Mem(Box::new(BinOp {
                    op    : Plus,
                    left  : Box::new(stack_frame),
                    right : Box::new(Const(pos)),
                }))
            },
            InReg(reg) => {
                Exp::Temp(reg)
            },
        }
    }

    fn external_call(name : &str, arguments : Vec<Exp>) -> Exp {
        Call(Box::new(Name(Label::with_name(name))), arguments)
    }
}