use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use position::WithPos;

pub type Symbol = i64;
pub type SymbolWithPos = WithPos<Symbol>;

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

    pub fn name(&self, symbol: Symbol) -> String {
        self.strings.strings.borrow()[&symbol].to_string()
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