use crate::op::*;
use crate::value::Value;

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn emit(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn emit_constant(&mut self, value: Value, line: usize) -> bool {
        if self.constants.len() < std::u8::MAX as usize {
            self.constants.push(value);
            self.emit(OP_CONSTANT, line);
            self.emit(self.constants.len() as u8 - 1, line);

            true
        } else {
            false
        }
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:04} ", self.lines[offset]);
        }
        match self.code[offset] {
            OP_CONSTANT => self.constant_instruction("OP_CONSTANT", offset),
            OP_NIL => simple_instruction("OP_NIL", offset),
            OP_TRUE => simple_instruction("OP_TRUE", offset),
            OP_FALSE => simple_instruction("OP_FALSE", offset),
            OP_EQUAL => simple_instruction("OP_EQUAL", offset),
            OP_GREATER => simple_instruction("OP_GREATER", offset),
            OP_LESS => simple_instruction("OP_LESS", offset),
            OP_ADD => simple_instruction("OP_ADD", offset),
            OP_SUBTRACT => simple_instruction("OP_SUBTRACT", offset),
            OP_MULTIPLY => simple_instruction("OP_MULTIPLY", offset),
            OP_DIVIDE => simple_instruction("OP_DIVIDE", offset),
            OP_NOT => simple_instruction("OP_NOT", offset),
            OP_NEGATE => simple_instruction("OP_NEGATE", offset),
            OP_RETURN => simple_instruction("OP_RETURN", offset),
            instruction => {
                println!("Unknown opcode: {}", instruction);
                offset + 1
            }
        }
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant = self.code[offset + 1];
        let value = &self.constants[constant as usize];
        println!("{:16} {:04} {}", name, constant, value);
        offset + 2
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{}", name);
    offset + 1
}
