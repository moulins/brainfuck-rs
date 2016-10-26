
use std::num::Wrapping;
use std::io::{stdin, Read};

pub type BfOffset = i32;
pub type BfValue = Wrapping<i8>;

pub struct Context {
  cells: Vec<BfValue>,
  cursor: usize,
  pc: usize
}

#[derive(Copy, Clone, Debug)]
pub enum OpCode {
  Halt,

  //Base instructions
  NoOp,
  Add(BfValue),
  Input,
  Output,
  JumpIfZero,
  JumpIfNonZero,

  //More instructions
  Set(BfValue)
}

#[derive(Copy, Clone, Debug)]
pub struct Instruction {
  pub opcode: OpCode,
  pub offset: BfOffset,
}

impl Context {
  pub fn new(size: usize) -> Self {
    assert!(size <= BfOffset::max_value() as usize, "the buffer is too large");

    Context {
      cells: vec![Wrapping(0); size],
      cursor: 0,
      pc: 0
    }
  }

  pub fn cur_cell(&self) -> BfValue {
    self.cells[self.cursor]
  }

  pub fn cur_cell_mut(&mut self) -> &mut BfValue {
    &mut self.cells[self.cursor]
  }

  pub fn move_cursor(&mut self, amount: BfOffset) {
    self.cursor += amount as usize;
  }

  pub fn jump(&mut self, val: BfOffset){
    self.pc = val as usize;
  }

  pub fn read(&self) -> BfValue {
    stdin().bytes().next().map_or(Wrapping(0), |val| {
      Wrapping(val.unwrap() as i8)
    })
  }

  pub fn write(&self, val: BfValue) {
    print!("{}", val.0 as u8 as char)
  }

  pub fn execute(&mut self, code: &[Instruction]) {
    let old = self.pc;
    self.pc = 0;
    while code[self.pc].execute(self) {
      self.pc += 1;
    }
    self.pc = old;
  }
}

impl Instruction {
  pub fn from(op: OpCode, offset: BfOffset) -> Self {
    Instruction {opcode: op, offset: offset}
  }

  pub fn from_op(op: OpCode) -> Self {
    Instruction::from(op, 0)
  }

  pub fn from_char(c: char) -> Self {
    match c {
      '+' => Instruction::from_op(OpCode::Add(Wrapping(1))),
      '-' => Instruction::from_op(OpCode::Add(Wrapping(-1))),
      '>' => Instruction::from(OpCode::NoOp, 1),
      '<' => Instruction::from(OpCode::NoOp, -1),
      '[' => Instruction::from_op(OpCode::JumpIfZero),
      ']' => Instruction::from_op(OpCode::JumpIfNonZero),
      '.' => Instruction::from_op(OpCode::Output),
      ',' => Instruction::from_op(OpCode::Input),
       _  => Instruction::from_op(OpCode::NoOp)
    }
  }

  pub fn use_offset(&self) -> bool {
    match self.opcode {
      OpCode::JumpIfZero | OpCode::JumpIfNonZero => true,
      _ => false
    }
  }

  fn execute(&self, ctx: &mut Context) -> bool {

    let mut offset = self.offset;

    match self.opcode {
      OpCode::Halt => return false,

      OpCode::NoOp => (), //Do nothing

      OpCode::Set(val) => {
        *ctx.cur_cell_mut() = val;
      },

      OpCode::Add(val) => {
        *ctx.cur_cell_mut() += val;
      },

      OpCode::Input => {
        *ctx.cur_cell_mut() = ctx.read();
      },

      OpCode::Output => {
        ctx.write(ctx.cur_cell());
      },

      OpCode::JumpIfZero => {
        if ctx.cur_cell() == Wrapping(0) {
          ctx.jump(self.offset);
        }
        offset = 0;
      },

      OpCode::JumpIfNonZero => {
        if ctx.cur_cell() != Wrapping(0) {
            ctx.jump(self.offset);
        }
        offset = 0;
      },
    }

    ctx.move_cursor(offset);

    true
  }
}