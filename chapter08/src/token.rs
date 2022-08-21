use std::fmt::{self, Display, Formatter};

use self::Tok::*;
use position::Pos;

#[derive(Clone, Debug)]
pub enum Tok {
    Ampersand,
    Array,
    Break,
    CloseCurly,
    CloseParen,
    CloseSquare,
    Colon,
    ColonEqual,
    Comma,
    Do,
    Dot,
    Else,
    End,
    Equal,
    For,
    Function,
    Greater,
    GreaterOrEqual,
    Ident(String),
    If,
    In,
    Int(i64),
    Lesser,
    LesserOrEqual,
    Let,
    Minus,
    Nil,
    NotEqual,
    Of,
    OpenCurly,
    OpenParen,
    OpenSquare,
    Pipe,
    Plus,
    Semicolon,
    Slash,
    Star,
    Str(String),
    Then,
    To,
    Type,
    Var,
    While,
}

#[derive(Debug)]
pub struct Token {
    pub pos: Pos,
    pub token: Tok,
}

impl Display for Tok {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let string = (|| {
            let string = match *self {
                Ampersand => "&",
                Array => "array",
                Break => "break",
                CloseCurly => "}",
                CloseParen => ")",
                CloseSquare => "]",
                Colon => ":",
                ColonEqual => ":=",
                Comma => ",",
                Do => "do",
                Dot => ".",
                Else => "else",
                Equal => "=",
                End => "end",
                For => "for",
                Function => "function",
                Greater => ">",
                GreaterOrEqual => ">=",
                Ident(ref ident) => ident,
                If => "if",
                In => "in",
                Int(num) => return num.to_string(),
                Lesser => "<",
                LesserOrEqual => "<=",
                Let => "let",
                Minus => "-",
                Nil => "nil",
                NotEqual => "<>",
                Of => "of",
                OpenCurly => "{",
                OpenParen => "(",
                OpenSquare => "[",
                Pipe => "|",
                Plus => "+",
                Semicolon => ";",
                Slash => "/",
                Star => "*",
                Str(ref string) => return format!("{:?}", string),
                Then => "then",
                To => "to",
                Type => "type",
                Var => "var",
                While => "while",
            };
            string.to_string()
        })();
        write!(formatter, "{}", string)
    }
}
