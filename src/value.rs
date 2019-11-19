use std::fmt;

#[derive(Copy, Clone)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
}

impl Value {
    pub fn is_bool(&self) -> bool {
        match self {
            Value::Bool(_) => true,
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            Value::Nil => true,
            _ => false,
        }
    }
    
    pub fn is_number(&self) -> bool {
        match self {
            Value::Number(_) => true,
            _ => false,
        }
    }

    pub fn equals(&self, value: &Value) -> bool {
        match (self, value) {
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
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

impl fmt::Display for Value {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Bool(x) => write!(f, "{}", x),
            Value::Nil => write!(f, "nil"),
            Value::Number(x) => write!(f, "{}", x),
        }
    }
}