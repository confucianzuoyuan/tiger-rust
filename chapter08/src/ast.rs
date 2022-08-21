use position::WithPos;
use symbol::{Symbol, SymbolWithPos};

#[derive(Clone, Debug)]
pub enum Declaration {
    Function(Vec<FuncDeclarationWithPos>),
    Type(Vec<TypeDecWithPos>),
    VariableDeclaration {
        escape: bool,
        init: ExprWithPos,
        name: Symbol,
        typ: Option<SymbolWithPos>,
    },
}

pub type DeclarationWithPos = WithPos<Declaration>;

#[derive(Clone, Debug)]
pub enum Expr {
    Array {
        init: Box<ExprWithPos>,
        size: Box<ExprWithPos>,
        typ: SymbolWithPos,
    },
    Assign {
        expr: Box<ExprWithPos>,
        var: VarWithPos,
    },
    Break,
    Call {
        args: Vec<ExprWithPos>,
        function: Symbol,
    },
    If {
        else_: Option<Box<ExprWithPos>>,
        test: Box<ExprWithPos>,
        then: Box<ExprWithPos>,
    },
    Int {
        value: i64,
    },
    Let {
        body: Box<ExprWithPos>,
        declarations: Vec<DeclarationWithPos>,
    },
    Nil,
    Oper {
        left: Box<ExprWithPos>,
        oper: OperationWithPos,
        right: Box<ExprWithPos>,
    },
    Record {
        fields: Vec<RecordFieldWithPos>,
        typ: SymbolWithPos,
    },
    Sequence(Vec<ExprWithPos>),
    Str {
        value: String,
    },
    Variable(VarWithPos),
    While {
        body: Box<ExprWithPos>,
        test: Box<ExprWithPos>,
    },
}

pub type ExprWithPos = WithPos<Expr>;

#[derive(Clone, Debug)]
pub struct Field {
    pub escape: bool,
    pub name: Symbol,
    pub typ: SymbolWithPos,
}

pub type FieldWithPos = WithPos<Field>;

#[derive(Clone, Debug)]
pub struct FuncDeclaration {
    pub body: ExprWithPos,
    pub name: Symbol,
    pub params: Vec<FieldWithPos>,
    pub result: Option<SymbolWithPos>,
}

pub type FuncDeclarationWithPos = WithPos<FuncDeclaration>;

#[derive(Clone, Copy, Debug)]
pub enum Operator {
    And,
    Divide,
    Equal,
    Ge,
    Gt,
    Le,
    Lt,
    Minus,
    Neq,
    Or,
    Plus,
    Times,
}

pub type OperationWithPos = WithPos<Operator>;

#[derive(Clone, Debug)]
pub struct RecordField {
    pub expr: ExprWithPos,
    pub ident: Symbol,
}

pub type RecordFieldWithPos = WithPos<RecordField>;

#[derive(Clone, Debug)]
pub enum Ty {
    Array { ident: SymbolWithPos },
    Name { ident: SymbolWithPos },
    Record { fields: Vec<FieldWithPos> },
}

#[derive(Clone, Debug)]
pub struct TypeDec {
    pub name: SymbolWithPos,
    pub ty: TyWithPos,
}

pub type TypeDecWithPos = WithPos<TypeDec>;

pub type TyWithPos = WithPos<Ty>;

#[derive(Clone, Debug)]
pub enum Var {
    Field {
        ident: SymbolWithPos,
        this: Box<VarWithPos>,
    },
    Simple {
        ident: SymbolWithPos,
    },
    Subscript {
        expr: Box<ExprWithPos>,
        this: Box<VarWithPos>,
    },
}

pub type VarWithPos = WithPos<Var>;

pub fn dummy_var_expr(symbol: Symbol) -> ExprWithPos {
    WithPos::dummy(Expr::Variable(WithPos::dummy(Var::Simple {
        ident: WithPos::dummy(symbol),
    })))
}
