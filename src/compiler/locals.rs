use std::rc::Rc;

use crate::scanner::Token;

pub struct Local {
    pub name: Rc<Token>,
    pub depth: i32,
}

impl Local {
    pub fn new(name: &Rc<Token>) -> Self {
        let name = Rc::clone(name);
        let depth = -1;
        Local { name, depth }
    }
}