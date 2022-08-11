use std::fmt::Debug;

use temp::Label;

pub mod x86_64;

/// 栈帧接口，不同的体系结构都可以实现这个接口
pub trait Frame: Clone {
    /// Access（访问方式）类型必须实现Clone和Debug
    type Access: Clone + Debug;

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
}