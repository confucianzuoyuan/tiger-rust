use std::collections::HashMap;

use super::Frame;
use asm::{Instruction, Subroutine};
use ir::BinOp::Plus;
use ir::Exp::{self, BinOp, Call, Const, Mem, Name};
use ir::Statement;
use temp::{Label, Temp};

use self::Access::{InFrame, InReg};

const POINTER_SIZE: i64 = 8;

#[derive(Clone)]
pub struct X86_64 {
    formals: Vec<Access>, // Representation of parameters.
    name: Label,
    pointer: i64,
}

impl PartialEq for X86_64 {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Clone, Debug)]
pub enum Access {
    InFrame(i64),
    InReg(Temp),
}

pub const RBP: Temp = Temp { num: 1 };
pub const RSP: Temp = Temp { num: 2 };
pub const RAX: Temp = Temp { num: 3 };
pub const RBX: Temp = Temp { num: 4 };
pub const RCX: Temp = Temp { num: 5 };
pub const RDX: Temp = Temp { num: 6 };
pub const RSI: Temp = Temp { num: 7 };
pub const RDI: Temp = Temp { num: 8 };
pub const R8: Temp = Temp { num: 9 };
pub const R9: Temp = Temp { num: 10 };
pub const R10: Temp = Temp { num: 11 };
pub const R11: Temp = Temp { num: 12 };
pub const R12: Temp = Temp { num: 13 };
pub const R13: Temp = Temp { num: 14 };
pub const R14: Temp = Temp { num: 15 };
pub const R15: Temp = Temp { num: 16 };

impl X86_64 {
    pub fn arg_registers() -> Vec<Temp> {
        vec![
            RDI,
            RSI,
            RDX,
            RCX,
            R8,
            R9,
        ]
    }

    fn callee_saved_registers() -> Vec<Temp> {
        vec![
            RBX,
            RBP,
            R12,
            R13,
            R14,
            R15,
        ]
    }

    fn special_registers() -> Vec<Temp> {
        vec![RAX, RSP]
    }

    fn caller_saved_registers() -> Vec<Temp> {
        vec![R10, R11]
    }

    pub fn calldefs() -> Vec<Temp> {
        let mut registers = Self::caller_saved_registers();
        registers.extend(Self::arg_registers());
        registers.push(Self::return_value());
        registers
    }
}

impl Frame for X86_64 {
    type Access = Access;

    const WORD_SIZE: i64 = 8;

    fn registers() -> Vec<Temp> {
        let mut registers = Self::arg_registers();
        registers.push(Self::return_value());
        registers.extend(Self::callee_saved_registers());
        registers.extend(Self::special_registers());
        registers.extend(Self::caller_saved_registers());
        registers
    }

    fn register_count() -> usize {
        Self::registers().len() - [RSP, RBP].len()
    }

    fn temp_map() -> HashMap<Temp, &'static str> {
        let mut map = HashMap::new();
        map.insert(RBP, "rbp");
        map.insert(RSP, "rsp");
        map.insert(RAX, "rax");
        map.insert(RBX, "rbx");
        map.insert(RDI, "rdi");
        map.insert(RSI, "rsi");
        map.insert(RDX, "rdx");
        map.insert(RCX, "rcx");
        map.insert(R8, "r8");
        map.insert(R9, "r9");
        map.insert(R10, "r10");
        map.insert(R11, "r11");
        map.insert(R12, "r12");
        map.insert(R13, "r13");
        map.insert(R14, "r14");
        map.insert(R15, "r15");
        map
    }

    fn special_name(temp: Temp) -> Option<&'static str> {
        Self::temp_map().get(&temp).map(|&str| str)
    }

    fn fp() -> Temp {
        RBP
    }

    fn return_value() -> Temp {
        RAX
    }

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

    fn name(&self) -> Label {
        self.name.clone()
    }

    fn formals(&self) -> &[Self::Access] {
        &self.formals
    }

    fn alloc_local(&mut self, escape: bool) -> Self::Access {
        if escape {
            self.pointer -= POINTER_SIZE;
            InFrame(self.pointer)
        } else {
            InReg(Temp::new())
        }
    }

    fn exp(&self, access: Self::Access, stack_frame: Exp) -> Exp {
        match access {
            InFrame(pos) => Mem(Box::new(BinOp {
                op: Plus,
                left: Box::new(stack_frame),
                right: Box::new(Const(pos)),
            })),
            InReg(reg) => Exp::Temp(reg),
        }
    }

    fn external_call(name: &str, arguments: Vec<Exp>) -> Exp {
        Call(Box::new(Name(Label::with_name(name))), arguments)
    }

    fn proc_entry_exit1(&mut self, mut statement: Statement) -> Statement {
        let mut start_statements = vec![];
        let mut end_statements = vec![];

        let mut saved_register_locations = vec![];
        for register in Self::callee_saved_registers().into_iter() {
            let local = Temp::new();
            let memory = Exp::Temp(local);
            saved_register_locations.push(memory.clone());
            start_statements.push(Statement::Move(memory, Exp::Temp(register)));
        }

        let arg_registers = Self::arg_registers();
        let arg_registers_len = arg_registers.len();
        for (formal, arg_register) in self.formals.iter().zip(arg_registers) {
            let destination = self.exp(formal.clone(), Exp::Temp(Self::fp()));
            start_statements.push(Statement::Move(destination, Exp::Temp(arg_register)));
        }
        for (index, formal) in self.formals.iter().skip(arg_registers_len).enumerate() {
            let destination = self.exp(formal.clone(), Exp::Temp(Self::fp()));
            start_statements.push(Statement::Move(
                destination,
                Exp::Mem(Box::new(Exp::BinOp {
                    left: Box::new(Exp::Temp(Self::fp())),
                    op: Plus,
                    right: Box::new(Exp::Const(Self::WORD_SIZE * (index + 2) as i64)),
                })),
            ));
        }

        for (register, location) in Self::callee_saved_registers()
            .into_iter()
            .zip(saved_register_locations)
        {
            end_statements.push(Statement::Move(Exp::Temp(register), location));
        }

        let mut end_statement = Statement::Exp(Exp::Const(0));
        for statement in end_statements {
            end_statement = Statement::Sequence(Box::new(end_statement), Box::new(statement));
        }

        for new_statement in start_statements.into_iter().rev() {
            statement = Statement::Sequence(Box::new(new_statement), Box::new(statement));
        }

        Statement::Sequence(Box::new(statement), Box::new(end_statement))
    }

    fn proc_entry_exit2(&self, mut instructions: Vec<Instruction>) -> Vec<Instruction> {
        let mut source = Self::callee_saved_registers();
        source.extend(Self::special_registers());
        let instruction = Instruction::Operation {
            assembly: String::new(),
            source,
            destination: vec![],
            jump: Some(vec![]),
        };
        instructions.push(instruction);

        for instruction in &mut instructions {
            match *instruction {
                Instruction::Label { .. } => (),
                Instruction::Move {
                    ref mut destination,
                    ..
                }
                | Instruction::Operation {
                    ref mut destination,
                    ..
                } => {
                    destination.push(RBP);
                    destination.push(RSP);
                    break;
                }
            }
        }

        for instruction in instructions.iter_mut().rev() {
            match *instruction {
                Instruction::Label { .. } => (),
                Instruction::Move { ref mut source, .. }
                | Instruction::Operation { ref mut source, .. } => {
                    source.push(RBP);
                    source.push(RSP);
                    break;
                }
            }
        }

        instructions
    }

    fn proc_entry_exit3(&self, body: Vec<Instruction>) -> Subroutine {
        let mut stack_size = -self.pointer;
        if stack_size % 16 != 0 {
            // Align the stack of 16 bytes.
            stack_size = (stack_size & !0xF) + 0x10;
        }

        Subroutine {
            // FIXME: saving to rbp is apparently not needed in 64-bit.
            prolog: format!(
                "{}:
    push rbp
    mov rbp, rsp
    sub rsp, {}",
                self.name(),
                stack_size
            ),
            body,
            epilog: "leave
    ret"
            .to_string(),
        }
    }
}
