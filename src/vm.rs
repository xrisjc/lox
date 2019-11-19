use crate::chunk::Chunk;
use crate::compiler;
use crate::op::*;
use crate::value::Value;

struct ValueStack {
    stack: Vec<Value>,
}

impl ValueStack {
    pub fn new() -> ValueStack {
        ValueStack {
            stack: Vec::new(),
        }
    }

    pub fn is_number(&self, distance: usize) -> bool {
        let offset = self.stack.len() - 1 - distance;
        self.stack[offset].is_number()
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

    pub fn pop(&mut self) -> Value {
        match self.stack.pop() {
            Some(x) => x,
            None => panic!("stack underflow"),
        }
    }

    pub fn pop_f64(&mut self) -> f64 {
        match self.stack.pop() {
            Some(Value::Number(x)) => x,
            None => panic!("stack underflow"),
            _ => panic!("pop_f64 called on non-number"),
        }
    }
}

pub enum InterpretError {
    Compile,
    Runtime,
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

    loop {
        print!("          ");
        for value in stack.stack.iter() {
            print!("[ {} ]", value);
        }
        println!("");
        chunk.disassemble_instruction(ip);

        let line = chunk.lines[ip];
        let op = read_byte!(chunk.code, ip);

        match op {
            OP_CONSTANT => {
                let constant_offset = read_byte!(chunk.code, ip) as usize;
                let constant = chunk.constants[constant_offset];
                stack.push(constant);
            }

            OP_NIL => stack.push(Value::Nil),
            OP_TRUE => stack.push(Value::Bool(true)),
            OP_FALSE => stack.push(Value::Bool(false)),

            OP_EQUAL => {
                let b = stack.pop();
                let a = stack.pop();
                stack.push_bool(a.equals(&b));
            }

            OP_GREATER if stack.is_number(0) && stack.is_number(1) => {
                let b = stack.pop_f64();
                let a = stack.pop_f64();
                stack.push_bool(a > b);
            }
            OP_LESS if stack.is_number(0) && stack.is_number(1) => {
                let b = stack.pop_f64();
                let a = stack.pop_f64();
                stack.push_bool(a < b);
            }

            OP_ADD if stack.is_number(0) && stack.is_number(1) => {
                let b = stack.pop_f64();
                let a = stack.pop_f64();
                stack.push_f64(a + b);
            }
            OP_SUBTRACT if stack.is_number(0) && stack.is_number(1) => {
                let b = stack.pop_f64();
                let a = stack.pop_f64();
                stack.push_f64(a - b);
            }
            OP_MULTIPLY if stack.is_number(0) && stack.is_number(1) => {
                let b = stack.pop_f64();
                let a = stack.pop_f64();
                stack.push_f64(a * b);
            }
            OP_DIVIDE if stack.is_number(0) && stack.is_number(1) => {
                let b = stack.pop_f64();
                let a = stack.pop_f64();
                stack.push_f64(a / b);
            }

            OP_NOT => {
                let a = stack.pop();
                stack.push(a.is_falsey());
            }

            OP_ADD | OP_SUBTRACT | OP_MULTIPLY | OP_DIVIDE => {
                eprintln!("[line {}] operands must be numbers", line);
                return Err(InterpretError::Runtime);
            }

            OP_NEGATE if stack.is_number(0) => {
                let a = stack.pop_f64();
                stack.push_f64(-a);
            }
            OP_NEGATE => {
                eprintln!("[line {}] operand must be a number", line);
                return Err(InterpretError::Runtime);
            }

            OP_RETURN => {
                let value = stack.pop();
                println!("{}", value);
                return Ok(());
            }
            _ => {}
        }
    }
}
