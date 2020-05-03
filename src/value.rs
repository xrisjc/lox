use std::fmt;
use std::rc::Rc;

use crate::object::Obj;

#[derive(PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    Obj(Rc<Obj>),
}

impl Value {
    pub fn is_number(&self) -> bool {
        match self {
            Value::Number(_) => true,
            _ => false,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Value::Obj(x) => x.is_string(),
            _ => false,
        }
    }

    pub fn is_falsey(&self) -> Value {
        let result = match self {
            Value::Bool(x) => !x,
            Value::Nil => true,
            _ => false,
        };
        Value::Bool(result)
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Bool(x) => Value::Bool(*x),
            Value::Nil => Value::Nil,
            Value::Number(x) => Value::Number(*x),
            Value::Obj(o) => {
                let o = (**o).clone();
                Value::Obj(Rc::new(o))
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Bool(x) => write!(f, "{}", x),
            Value::Nil => write!(f, "nil"),
            Value::Number(x) => write!(f, "{}", x),
            Value::Obj(x) => write!(f, "{}", x),
        }
    }
}
