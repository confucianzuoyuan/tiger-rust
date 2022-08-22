use self::Type::*;
use symbol::{Symbol, SymbolWithPos, Symbols};

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Int,
    String,
    // 通过判断Unique是否相等来判断是否是相同的Record类型
    // 加快判断速度
    Record(Symbol, Vec<(Symbol, Type)>, Unique),
    Array(Box<Type>, Unique),
    Nil,
    Unit,
    Name(SymbolWithPos, Option<Box<Type>>),
    Error,
}

impl Type {
    pub fn show(&self, symbols: &Symbols<()>) -> std::string::String {
        match *self {
            Array(ref typ, _) => {
                format!("[{}]", typ.show(symbols))
            }
            Int => "int".to_string(),
            Name(_, ref typ) => {
                if let Some(typ) = typ {
                    typ.show(symbols)
                } else {
                    "unresolved type".to_string()
                }
            }
            Nil => "nil".to_string(),
            Record(name, _, _) => format!("struct {}", symbols.name(name)),
            String => "string".to_string(),
            Unit => "()".to_string(),
            Error => "type error".to_string(),
        }
    }
}

static mut UNIQUE_COUNT: u64 = 0;

#[derive(Clone, Debug, PartialEq)]
pub struct Unique(u64);

impl Unique {
    pub fn new() -> Self {
        let value = unsafe { UNIQUE_COUNT };
        unsafe {
            UNIQUE_COUNT += 1;
        }
        Unique(value)
    }
}
