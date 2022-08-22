use std::fmt::{self, Display, Formatter};

use self::Label::{Named, Num};
use frame::Frame;

/// 临时变量（虚拟寄存器）类型
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Temp {
    num: u32,
}

/// 实例化临时变量，使用计数器的方式
impl Temp {
    pub fn new() -> Self {
        static mut COUNTER: u32 = 0;
        unsafe {
            COUNTER += 1;
            Self { num: COUNTER }
        }
    }

    pub fn to_string<F: Frame>(&self) -> String {
        F::special_name(*self)
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("t{}", self.num))
    }
}

/// 标签主要分为两种
///   - 有名字的标签：如"END"这种
///   - 没有名字的标签：如"L11"这种
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Label {
    Named(String),
    Num(u32),
}

/// 标签的实现
impl Label {
    /// 实例化一个没有名字的标签，使用一个计数器，
    /// 每实例化一个标签，计数器就加一
    pub fn new() -> Self {
        static mut COUNTER: u32 = 0;
        unsafe {
            COUNTER += 1;
            Num(COUNTER)
        }
    }

    pub fn to_name(&self) -> String {
        match *self {
            Named(ref name) => name.clone(),
            Num(_) => panic!("Expected Named"),
        }
    }

    /// 实例化一个有名字的标签
    pub fn with_name(name: &str) -> Self {
        Named(name.to_string())
    }
}

/// 打印标签
impl Display for Label {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            Named(ref name) => write!(formatter, "{}", name),
            Num(num) => write!(formatter, "l{}", num),
        }
    }
}
