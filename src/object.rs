use std::fmt;
use std::rc::Rc;

#[derive(Clone, PartialEq)]
pub enum ObjValue {
    String(String),
}

impl ObjValue {
    pub fn is_string(&self) -> bool {
        match self {
            ObjValue::String(_) => true,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            ObjValue::String(s) => Some(&s),
        }
    }
}

impl fmt::Display for ObjValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ObjValue::String(x) => write!(f, "{}", x),
        }
    }
}

#[derive(Clone)]
pub struct Obj {
    pub value: ObjValue,
}

impl Obj {
    pub fn new_string(s: String) -> Rc<Obj> {
        let value = ObjValue::String(s);
        let obj = Obj { value };
        Rc::new(obj)
    }

    pub fn is_string(&self) -> bool {
        self.value.is_string()
    }

    pub fn as_str(&self) -> Option<&str> {
        self.value.as_str()
    }
}

impl PartialEq for Obj {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
