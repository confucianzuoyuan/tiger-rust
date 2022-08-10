use std::collections::HashMap;
use std::rc::Rc;

use symbol::{Strings, Symbol, Symbols};
use types::Type;

#[derive(Clone)]
pub enum Entry {
    Fun {
        parameters: Vec<Type>,
        result: Type,
    },
    Var {
        typ: Type,
    },
    Error,
}

pub struct Env {
    type_env: Symbols<Type>,
    var_env: Symbols<Entry>,
}

impl Env {
    pub fn new(strings: &Rc<Strings>) -> Self {
        let mut type_env = Symbols::new(Rc::clone(strings));
        let int_symbol = type_env.symbol("int");
        type_env.enter(int_symbol, Type::Int);
        let string_symbol = type_env.symbol("string");
        type_env.enter(string_symbol, Type::String);

        let var_env = Symbols::new(Rc::clone(strings));
        let mut env = Self {
            type_env,
            var_env,
        };

        for (name, (param_types, return_type)) in external_functions() {
            env.add_function(name, param_types, return_type);
        }

        env
    }

    fn add_function(&mut self, name: &str, parameters: Vec<Type>, result: Type) {
        let symbol = self.var_env.symbol(name);
        let entry = Entry::Fun {
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

    pub fn enter_type(&mut self, symbol: Symbol, typ: Type) {
        self.type_env.enter(symbol, typ);
    }

    pub fn enter_var(&mut self, symbol: Symbol, data: Entry) {
        self.var_env.enter(symbol, data);
    }

    pub fn look_type(&self, symbol: Symbol) -> Option<&Type> {
        self.type_env.look(symbol)
    }

    pub fn look_var(&mut self, symbol: Symbol) -> Option<&Entry> {
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