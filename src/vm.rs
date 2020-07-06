use crate::chunk::Chunk;
use crate::compiler;
use crate::object::Obj;
use crate::op::*;
use crate::value::Value;

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub enum InterpretError {
    Compile,
    Runtime,
}

impl Error for InterpretError {}

impl fmt::Display for InterpretError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
        ValueStack { stack: Vec::new() }
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

    pub fn push_offset(&mut self, offset: usize) {
        let value = self.stack[offset].clone();
        self.push(value);
    }

    pub fn pop(&mut self) -> Result<Value, InterpretError> {
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

    pub fn dup_to(&mut self, offset: usize) {
        let top_offset = self.stack.len() - 1;
        let top_value = self.stack[top_offset].clone();
        self.stack[offset] = top_value;
    }

    pub fn print(&self) {
        print!("          ");
        for value in self.stack.iter() {
            print!("[ {} ]", value);
        }
        println!("");
    }
}

pub fn interpret(source: &str, globals: &mut HashMap<String, Value>) -> Result<(), InterpretError> {
    let mut chunk = Chunk::new();
    if compiler::compile(source, &mut chunk) {
        run(&chunk, globals)
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

macro_rules! read_constant {
    ($code:expr, $ip:expr, $constants:expr) => {{
        let constant_offset = read_byte!($code, $ip) as usize;
        &$constants[constant_offset]
    }};
}

macro_rules! read_string {
    ($code:expr, $ip:expr, $constants:expr) => {{
        read_constant!($code, $ip, $constants)
            .as_str()
            .expect("expected string constant for a global variable")
            .to_owned()
    }};
}

fn run(chunk: &Chunk, globals: &mut HashMap<String, Value>) -> Result<(), InterpretError> {
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
                let constant = read_constant!(chunk.code, ip, chunk.constants);
                let constant = constant.clone();
                if let Value::Obj(obj) = &constant {
                    objects.push(Rc::clone(obj));
                }
                stack.push(constant);
            }

            OP_NIL => stack.push(Value::Nil),
            OP_TRUE => stack.push(Value::Bool(true)),
            OP_FALSE => stack.push(Value::Bool(false)),

            OP_POP => {
                stack.pop()?;
            }

            OP_GET_LOCAL => {
                let slot = read_byte!(chunk.code, ip);
                stack.push_offset(slot as usize);
            }

            OP_SET_LOCAL => {
                let slot = read_byte!(chunk.code, ip);
                stack.dup_to(slot as usize);
            }

            OP_GET_GLOBAL => {
                let key = read_string!(chunk.code, ip, chunk.constants);
                match globals.get(&key) {
                    Some(value) => stack.push(value.clone()),
                    None => {
                        let message = format!("Undefined variable '{}'.", key);
                        return runtime_error(&message);
                    }
                }
            }

            OP_DEFINE_GLOBAL => {
                let key = read_string!(chunk.code, ip, chunk.constants);
                let value = stack.peek(0).clone();
                globals.insert(key, value);

                stack.pop()?;
            }

            OP_SET_GLOBAL => {
                let key = read_string!(chunk.code, ip, chunk.constants);
                if globals.contains_key(&key) {
                    let value = stack.peek(0).clone();
                    globals.insert(key, value);
                } else {
                    let message = format!("Undefined variable '{}'.", key);
                    return runtime_error(&message);   
                }
            }

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

                let mut s = a.as_str().unwrap().to_owned();
                s.push_str(b.as_str().unwrap());

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

            OP_PRINT => {
                let value = stack.pop()?;
                println!("{}", value);
            }

            OP_RETURN => {
                return Ok(());
                // let value = stack.pop()?;
                // println!("{}", value);
            }

            _ => {
                return runtime_error("unknown op");
            }
        }
    }
}
