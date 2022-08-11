use std::cell::RefCell;
use std::rc::Rc;

use frame::Frame;
use temp::Label;

#[allow(type_alias_bounds)]
pub type Access<F: Frame> = (Level<F>, F::Access);

pub struct Level<F> {
    pub current: Rc<RefCell<F>>,
    parent: Option<Box<Level<F>>>,
}

impl<F> Clone for Level<F> {
    fn clone(&self) -> Self {
        Self {
            current: self.current.clone(),
            parent: self.parent.clone(),
        }
    }
}

impl<F: PartialEq> PartialEq for Level<F> {
    fn eq(&self, other: &Self) -> bool {
        self.current == other.current
    }
}

pub fn outermost<F: Frame>() -> Level<F> {
    Level {
        current: Rc::new(RefCell::new(F::new(Label::new(), vec![]))),
        parent: None,
    }
}

impl<F: Frame> Level<F> {
    pub fn new(parent: &Level<F>, name: Label, mut formals: Vec<bool>) -> Level<F> {
        formals.push(true);
        Level {
            current: Rc::new(RefCell::new(F::new(name, formals))),
            parent: Some(Box::new(parent.clone())),
        }
    }

    pub fn formals(&self) -> Vec<Access<F>> {
        self.current
            .borrow()
            .formals()
            .iter()
            .map(|access| (self.clone(), access.clone()))
            .collect()
    }
}

pub fn alloc_local<F: Frame>(level: &Level<F>, escape: bool) -> Access<F> {
    let level = level.clone();
    let frame_local =
        level.current
             .borrow_mut()
             .alloc_local(escape);
    (level, frame_local)
}