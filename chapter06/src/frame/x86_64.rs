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

/// X86_64的栈帧结构
impl Frame for X86_64 {
    type Access = Access;

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
}