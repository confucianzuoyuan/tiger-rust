use std::fmt::{self, Display, Formatter};

use frame::Frame;
use self::Label::{Named, Num};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Temp {
    pub num: u32,
}

impl Temp {
    pub fn new() -> Self {
        static mut COUNTER: u32 = 16;
        unsafe {
            COUNTER += 1;
            Self {
                num: COUNTER,
            }
        }
    }

    pub fn to_string<F: Frame>(&self) -> String {
        F::special_name(*self)
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("t{}", self.num))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Label {
    Named(String),
    Num(u32),
}

impl Label {
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

    pub fn with_name(name: &str) -> Self {
        Named(name.to_string())
    }
}

impl Display for Label {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            Named(ref name) => write!(formatter, "{}", name),
            Num(num) => write!(formatter, "l{}", num),
        }
    }
}
