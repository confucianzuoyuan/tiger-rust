use std::collections::HashMap;
use std::rc::Rc;

use escape::EscapeEnv;
use frame::Frame;
use gen;
use gen::{Access, Level};
use symbol::{Strings, Symbol, Symbols};
use temp::Label;
use types::Type;

#[derive(Clone)]
pub enum Entry<F: Clone + Frame> {
    Fun {
        external: bool,
        label: Label,
        level: Level<F>,
        parameters: Vec<Type>,
        result: Type,
    },
    Var {
        access: Access<F>,
        typ: Type,
    },
    Error,
}

/// 三种环境
///   - 逃逸环境
///   - 类型环境
///   - 值环境
pub struct Env<F: Clone + Frame> {
    escape_env: EscapeEnv,
    type_env: Symbols<Type>,
    var_env: Symbols<Entry<F>>,
}

impl<F: Clone + Frame> Env<F> {
    pub fn new(strings: &Rc<Strings>, escape_env: EscapeEnv) -> Self {
        let mut type_env = Symbols::new(Rc::clone(strings));
        let int_symbol = type_env.symbol("int");
        type_env.enter(int_symbol, Type::Int);
        let string_symbol = type_env.symbol("string");
        type_env.enter(string_symbol, Type::String);

        let var_env = Symbols::new(Rc::clone(strings));
        let mut env = Self {
            escape_env,
            type_env,
            var_env,
        };

        for (name, (param_types, return_type)) in external_functions() {
            env.add_function(name, param_types, return_type);
        }

        env
    }

    /// 负责添加系统函数
    /// 系统函数一定是最外层函数
    fn add_function(&mut self, name: &str, parameters: Vec<Type>, result: Type) {
        let symbol = self.var_env.symbol(name);
        let entry = Entry::Fun {
            external: true,
            label: Label::with_name(name),
            level: gen::outermost(),
            parameters,
            result,
        };
        self.var_env.enter(symbol, entry);
    }

    pub fn begin_scope(&mut self) {
        self.type_env.begin_scope();
        self.var_env.begin_scope();
    }

    pub fn end_scope(&mut self) {
        self.type_env.end_scope();
        self.var_env.end_scope();
    }

    /// type a = int
    /// 将a -> int键值对写入类型环境中
    pub fn enter_type(&mut self, symbol: Symbol, typ: Type) {
        self.type_env.enter(symbol, typ);
    }

    /// var a : int := 1
    /// 将a -> int键值对写入值环境
    pub fn enter_var(&mut self, symbol: Symbol, data: Entry<F>) {
        self.var_env.enter(symbol, data);
    }

    /// 查找符号的逃逸信息
    pub fn look_escape(&self, symbol: Symbol) -> bool {
        self.escape_env
            .look(symbol)
            .expect("escape")
            .escape
    }

    /// 查找符号是哪种类型的别名
    /// type a = int
    pub fn look_type(&self, symbol: Symbol) -> Option<&Type> {
        self.type_env.look(symbol)
    }

    /// 查找变量的类型，是int还是函数之类的类型
    pub fn look_var(&mut self, symbol: Symbol) -> Option<&Entry<F>> {
        self.var_env.look(symbol)
    }

    pub fn replace_type(&mut self, symbol: Symbol, typ: Type) {
        self.type_env.replace(symbol, typ);
    }

    pub fn type_name(&self, symbol: Symbol) -> String {
        self.type_env.name(symbol)
    }

    pub fn var_name(&self, symbol: Symbol) -> String {
        self.var_env.name(symbol)
    }
}

/// &'static表示生命周期是整个程序的运行过程
pub fn external_functions() -> HashMap<&'static str, (Vec<Type>, Type)> {
    let mut functions = HashMap::new();
    functions.insert("print", (vec![Type::String], Type::Unit));
    functions.insert("printi", (vec![Type::Int], Type::Unit));
    functions.insert("flush", (vec![], Type::Unit));
    functions.insert("getchar", (vec![], Type::String));
    functions.insert("ord", (vec![Type::String], Type::Int));
    functions.insert("chr", (vec![Type::Int], Type::String));
    functions.insert("size", (vec![Type::String], Type::Int));
    functions.insert("substring", (vec![Type::String, Type::Int, Type::Int], Type::String));
    functions.insert("concat", (vec![Type::String, Type::String], Type::String));
    functions.insert("not", (vec![Type::Int], Type::Int));
    functions.insert("exit", (vec![Type::Int], Type::Unit));
    functions.insert("stringEqual", (vec![Type::String, Type::String], Type::Int));

    functions.insert("malloc", (vec![Type::Int], Type::Int));
    functions.insert("initArray", (vec![Type::Int, Type::Int], Type::Int));
    functions
}