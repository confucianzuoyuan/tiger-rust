use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use position::WithPos;

pub type Symbol = i64;
pub type SymbolWithPos = WithPos<Symbol>;

/// 符号的结构体
#[derive(Debug)]
pub struct Strings {
    next_symbol: RefCell<Symbol>,
    strings: RefCell<HashMap<Symbol, String>>,
}

impl Strings {
    pub fn new() -> Self {
        Self {
            next_symbol: RefCell::new(0),
            strings: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(&self, symbol: Symbol) -> Option<String> {
        let strings = self.strings.borrow();
        strings.get(&symbol).map(Clone::clone)
    }
}

#[derive(Debug)]
pub struct Symbols<T> {
    stack: Vec<Vec<Symbol>>,
    strings: Rc<Strings>,
    table: HashMap<Symbol, Vec<T>>,
}

impl<T> Symbols<T> {
    pub fn new(strings: Rc<Strings>) -> Self {
        let mut symbols = Self {
            stack: vec![],
            strings,
            table: HashMap::new(),
        };
        symbols.begin_scope();
        symbols
    }

    pub fn begin_scope(&mut self) {
        self.stack.push(vec![]);
    }

    pub fn end_scope(&mut self) {
        for symbol in self.stack.last().expect("Call begin_scope() before end_scope()") {
            let bindings = self.table.get_mut(symbol).expect("Symbol not in table");
            bindings.pop();
        }
        self.stack.pop();
    }

    pub fn enter(&mut self, symbol: Symbol, data: T) {
        let bindings = self.table.entry(symbol).or_insert_with(Vec::new);
        bindings.push(data);
        let current_bindings = self.stack.last_mut().expect("Call begin_scope() before enter()");
        current_bindings.push(symbol);
    }

    pub fn look(&self, symbol: Symbol) -> Option<&T> {
        self.table.get(&symbol).and_then(|vec| vec.last())
    }

    pub fn look_mut(&mut self, symbol: Symbol) -> Option<&mut T> {
        self.table
            .get_mut(&symbol)
            .and_then(|vec| vec.last_mut())
    }

    pub fn name(&self, symbol: Symbol) -> String {
        self.strings.strings.borrow()[&symbol].to_string()
    }

    pub fn replace(&mut self, symbol: Symbol, data: T) {
        let bindings = self.table.entry(symbol).or_insert_with(Vec::new);
        bindings.pop().expect("Call enter() before replace()");
        bindings.push(data);
    }

    pub fn symbol(&mut self, string: &str) -> Symbol {
        if let Some((&key, _)) = self.strings.strings.borrow().iter().find(|&(_, value)| value == string) {
            return key;
        }

        let symbol = *self.strings.next_symbol.borrow();
        self.strings.strings.borrow_mut().insert(symbol, string.to_string());
        *self.strings.next_symbol.borrow_mut() += 1;
        symbol
    }
}