use crate::chunk::Chunk;
use crate::compiler;
use crate::object::Obj;
use crate::op::*;
use crate::value::Value;

use std::error::Error;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub enum InterpretError {
    Compile,
    Runtime,
}

impl Error for InterpretError { }

impl fmt::Display for InterpretError {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InterpretError::Compile => write!(f, "compile failed"), 
            InterpretError::Runtime => write!(f, "runtime error"), 
        }
    }
}

fn runtime_error<T>(msg: &str) -> Result<T, InterpretError> {
    eprintln!("runtime error: {}", msg);
    Err(InterpretError::Runtime)
}

struct ValueStack {
    stack: Vec<Value>,
}

impl ValueStack {
    pub fn new() -> ValueStack {
        ValueStack {
            stack: Vec::new(),
        }
    }

    pub fn peek(&self, distance: usize) -> &Value {
        let offset = self.stack.len() - 1 - distance;
        &self.stack[offset]
    }

    pub fn is_number(&self, distance: usize) -> bool {
        self.peek(distance).is_number()
    }

    pub fn is_string(&self, distance: usize) -> bool {
        self.peek(distance).is_string()
    }

    pub fn push(&mut self, x: Value) {
        self.stack.push(x);
    }

    pub fn push_f64(&mut self, x: f64) {
        self.stack.push(Value::Number(x));
    }

    pub fn push_bool(&mut self, x: bool) {
        self.stack.push(Value::Bool(x));
    }

    pub fn pop(&mut self) -> Result<Value, InterpretError>  {
        match self.stack.pop() {
            Some(x) => Ok(x),
            None => runtime_error("stack underflow"),
        }
    }

    pub fn pop_f64(&mut self) -> Result<f64, InterpretError> {
        match self.stack.pop() {
            Some(Value::Number(x)) => Ok(x),
            None => runtime_error("stack underflow"),
            _ => runtime_error("pop_f64 called on non-number"),
        }
    }

    pub fn pop_obj(&mut self) -> Result<Rc<Obj>, InterpretError> {
        match self.stack.pop() {
            Some(Value::Obj(x)) => Ok(Rc::clone(&x)),
            None => runtime_error("stack underflow"),
            _ => runtime_error("pop_obj called on non-obj"),
        }
    }

    pub fn print(&self) {
        print!("          ");
        for value in self.stack.iter() {
            print!("[ {} ]", value);
        }
        println!("");
    }
}

pub fn interpret(source: &str) -> Result<(), InterpretError> {
    let mut chunk = Chunk::new();
    if compiler::compile(source, &mut chunk) {
        run(&chunk)
    } else {
        Err(InterpretError::Compile)
    }
}

macro_rules! read_byte {
    ($code:expr, $ip:expr) => {{
        let byte = $code[$ip];
        $ip += 1;
        byte
    }};
}

fn run(chunk: &Chunk) -> Result<(), InterpretError> {
    if chunk.code.len() == 0 {
        return Ok(());
    }

    let mut ip = 0;
    let mut stack = ValueStack::new();
    let mut objects = Vec::new();

    loop {
        stack.print();
        chunk.disassemble_instruction(ip);

        let line = chunk.lines[ip];
        let op = read_byte!(chunk.code, ip);

        match op {
            OP_CONSTANT => {
                let constant_offset = read_byte!(chunk.code, ip) as usize;
                let constant = &chunk.constants[constant_offset];
                let constant = constant.clone();
                if let Value::Obj(obj) = &constant {
                    objects.push(Rc::clone(obj));
                }
                stack.push(constant);
            }

            OP_NIL => stack.push(Value::Nil),
            OP_TRUE => stack.push(Value::Bool(true)),
            OP_FALSE => stack.push(Value::Bool(false)),

            OP_EQUAL => {
                let b = stack.pop()?;
                let a = stack.pop()?;
                stack.push_bool(a == b);
            }

            OP_GREATER if stack.is_number(0) && stack.is_number(1) => {
                let b = stack.pop_f64()?;
                let a = stack.pop_f64()?;
                stack.push_bool(a > b);
            }
            OP_LESS if stack.is_number(0) && stack.is_number(1) => {
                let b = stack.pop_f64()?;
                let a = stack.pop_f64()?;
                stack.push_bool(a < b);
            }
            OP_ADD if stack.is_string(0) && stack.is_string(1) => {
                let b = stack.pop_obj()?;
                let a = stack.pop_obj()?;

                let mut s = String::from(a.as_str());
                s.push_str(b.as_str());

                let s = Obj::new_string(s);
                objects.push(Rc::clone(&s));
                let value = Value::Obj(s);

                stack.push(value);
            }
            OP_ADD if stack.is_number(0) && stack.is_number(1) => {
                let b = stack.pop_f64()?;
                let a = stack.pop_f64()?;
                stack.push_f64(a + b);
            }
            OP_SUBTRACT if stack.is_number(0) && stack.is_number(1) => {
                let b = stack.pop_f64()?;
                let a = stack.pop_f64()?;
                stack.push_f64(a - b);
            }
            OP_MULTIPLY if stack.is_number(0) && stack.is_number(1) => {
                let b = stack.pop_f64()?;
                let a = stack.pop_f64()?;
                stack.push_f64(a * b);
            }
            OP_DIVIDE if stack.is_number(0) && stack.is_number(1) => {
                let b = stack.pop_f64()?;
                let a = stack.pop_f64()?;
                stack.push_f64(a / b);
            }

            OP_NOT => {
                let a = stack.pop()?;
                stack.push(a.is_falsey());
            }

            OP_ADD | OP_SUBTRACT | OP_MULTIPLY | OP_DIVIDE => {
                let msg = format!("[line {}] operands must be numbers", line);
                return runtime_error(&msg);
            }

            OP_NEGATE if stack.is_number(0) => {
                let a = stack.pop_f64()?;
                stack.push_f64(-a);
            }
            OP_NEGATE => {
                let msg = format!("[line {}] operand must be a number", line);
                return runtime_error(&msg);
            }

            OP_RETURN => {
                let value = stack.pop()?;
                println!("{}", value);
                return Ok(());
            }
            _ => {
                return runtime_error("unknown op");
            }
        }
    }
}
