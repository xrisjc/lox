use crate::op::*;
use crate::value::Value;

// Maximum number of constants allowed in a chunk.  A constant index must fit
// in a byte.
const MAX_CONSTANTS: usize = std::u8::MAX as usize;

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

    /// Adds a value to the chunk's constant table.  Returns the value's index
    /// in the constant table.
    pub fn add_constant(&mut self, value: Value) -> Result<u8, String> {
        if self.constants.len() < MAX_CONSTANTS {
            self.constants.push(value);
            let index = self.constants.len() as u8 - 1;
            Ok(index)
        } else {
            let message = String::from("Too many constants in one chunk.");
            Err(message)
        }
    }

    pub fn emit(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn emit_constant(&mut self, value: Value, line: usize) -> Result<u8, String> {
        let index = self.add_constant(value)?;
        self.emit(OP_CONSTANT, line);
        self.emit(index, line);
        Ok(index)
    }

    pub fn emit_jump(&mut self, instruction: u8, line: usize) -> usize {
        self.emit(instruction, line);
        self.emit(0xff, line);
        self.emit(0xff, line);
        
        self.code.len() - 2
    }


    pub fn patch_jump(&mut self, offset: usize) -> Result<(), String> {
        // -2 to adjust for the bytecode for the jump offset itself.
        let jump = self.code.len() - offset - 2;
        let max_jump = std::u16::MAX as usize;

        if jump > max_jump {
            return Err(String::from("Too much code to jump over."));
        }

        self.code[offset] = ((jump >> 8) & 0xff) as u8;
        self.code[offset + 1] = (jump & 0xff) as u8;

        Ok(())
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
            OP_POP => simple_instruction("OP_POP", offset),
            OP_GET_LOCAL => self.byte_instruction("OP_GET_LOCAL", offset),
            OP_SET_LOCAL => self.byte_instruction("OP_SET_LOCAL", offset),
            OP_GET_GLOBAL => self.constant_instruction("OP_GET_GLOBAL", offset),
            OP_DEFINE_GLOBAL => self.constant_instruction("OP_DEFINE_GLOBAL", offset),
            OP_SET_GLOBAL => self.constant_instruction("OP_SET_GLOBAL", offset),
            OP_EQUAL => simple_instruction("OP_EQUAL", offset),
            OP_GREATER => simple_instruction("OP_GREATER", offset),
            OP_LESS => simple_instruction("OP_LESS", offset),
            OP_ADD => simple_instruction("OP_ADD", offset),
            OP_SUBTRACT => simple_instruction("OP_SUBTRACT", offset),
            OP_MULTIPLY => simple_instruction("OP_MULTIPLY", offset),
            OP_DIVIDE => simple_instruction("OP_DIVIDE", offset),
            OP_NOT => simple_instruction("OP_NOT", offset),
            OP_NEGATE => simple_instruction("OP_NEGATE", offset),
            OP_PRINT => simple_instruction("OP_PRINT", offset),
            OP_JUMP => self.jump_instruction("OP_JUMP", 1, offset),
            OP_JUMP_IF_FALSE => self.jump_instruction("OP_JUMP_IF_FALSE", 1, offset),
            OP_RETURN => simple_instruction("OP_RETURN", offset),
            instruction => {
                println!("Unknown opcode: {}", instruction);
                offset + 1
            }
        }
    }

    fn byte_instruction(&self, name: &str, offset: usize) -> usize {
        let slot = self.code[offset + 1];
        println!("{:16} {:04}", name, slot);
        offset + 2
    }

    fn jump_instruction(&self, name: &str, sign: i32, offset: usize) -> usize {
        let hi = self.code[offset + 1] as usize;
        let lo = self.code[offset + 2] as usize;
        let jump = (hi << 8) | lo;
        let jump = if sign < 0 { offset + 3 + jump } else { offset + 3 - jump };
        println!("{:16} {:04} {}", name, offset, jump);
        offset + 3
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
