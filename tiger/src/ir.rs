#![allow(dead_code)]

use temp::{Label, Temp};

#[derive(Clone, Debug, PartialEq)]
pub enum Exp {
    Const(i64),
    /// Dummy expression to return when there is an error.
    Error,
    Name(Label),
    Temp(Temp),
    BinOp {
        op: BinOp,
        left: Box<Exp>,
        right: Box<Exp>,
    },
    Mem(Box<Exp>),
    Call(Box<Exp>, Vec<Exp>),
    ExpSequence(Box<Statement>, Box<Exp>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Move(Exp, Exp),
    Exp(Exp),
    Jump(Exp, Vec<Label>),
    CondJump {
        op: RelationalOp,
        left: Exp,
        right: Exp,
        true_label: Label,
        false_label: Label,
    },
    Sequence(Box<Statement>, Box<Statement>),
    Label(Label),
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinOp {
    Plus,
    Minus,
    Mul,
    Div,
    And,
    Or,
    ShiftLeft,
    ShiftRight,
    ArithmeticShiftRight,
    Xor,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RelationalOp {
    Equal,
    NotEqual,
    LesserThan,
    GreaterThan,
    LesserOrEqual,
    GreaterOrEqual,
    UnsignedLesserThan,
    UnsignedLesserOrEqual,
    UnsignedGreaterThan,
    UnsignedGreaterOrEqual,
}
