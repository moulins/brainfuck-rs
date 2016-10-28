
use std::num::Wrapping;
use std::io::{stdin, Read};

pub type BfOffset = i32;
pub type BfValue = i8;


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
  Set(BfValue),
  FindZero(BfValue),
  MoveAndAddTo(BfValue),
}

#[derive(Copy, Clone, Debug)]
pub struct Instruction {
  pub opcode: OpCode,
  pub offset: BfOffset,
}

impl Instruction {
  #[inline]
  pub fn from(op: OpCode, offset: BfOffset) -> Self {
    Instruction {opcode: op, offset: offset}
  }

  #[inline]
  pub fn from_op(op: OpCode) -> Self {
    Instruction::from(op, 0)
  }

  pub fn from_char(c: char) -> Self {
    match c {
      '+' => Instruction::from_op(OpCode::Add(1)),
      '-' => Instruction::from_op(OpCode::Add(-1)),
      '>' => Instruction::from(OpCode::NoOp, 1),
      '<' => Instruction::from(OpCode::NoOp, -1),
      '[' => Instruction::from_op(OpCode::JumpIfZero),
      ']' => Instruction::from_op(OpCode::JumpIfNonZero),
      '.' => Instruction::from_op(OpCode::Output),
      ',' => Instruction::from_op(OpCode::Input),
       _  => Instruction::from_op(OpCode::NoOp)
    }
  }

  #[inline]
  pub fn use_offset(&self) -> bool {
    match self.opcode {
      OpCode::NoOp | OpCode::JumpIfZero | OpCode::JumpIfNonZero => true,
      _ => false
    }
  }

  #[inline]
  pub fn offset_to_value(&self) -> Option<BfValue> {
    if self.offset == (self.offset as BfValue as BfOffset) {
    Some(self.offset as BfValue)
    } else { None }
  }

  fn execute(&self, ctx: &mut Context) -> bool {

    let mut offset = self.offset;

    //println!("{:?}", self);
    match self.opcode {
      OpCode::Halt => return false,

      OpCode::NoOp => (), //Do nothing

      OpCode::Set(val) => {
        *ctx.cell_mut() = val;
      },

      OpCode::Add(val) => {
        *ctx.cell_mut() = ctx.cell().wrapping_add(val);
      },

      OpCode::MoveAndAddTo(offset) => {
        if ctx.cell() != 0 {
          let o = offset as BfOffset;
          *ctx.cell_at_mut(o) = ctx.cell_at(o).wrapping_add(ctx.cell());
          *ctx.cell_mut() = 0;
        }
      }

      OpCode::Input => {
        *ctx.cell_mut() = ctx.read();
      },

      OpCode::Output => {
        ctx.write(ctx.cell());
      },

      OpCode::JumpIfZero => {
        if ctx.cell() == 0 {
          ctx.jump(self.offset);
        }
        offset = 0;
      },

      OpCode::JumpIfNonZero => {
        if ctx.cell() != 0 {
            ctx.jump(self.offset);
        }
        offset = 0;
      },

      OpCode::FindZero(step) => {
        while ctx.cell() != 0 {
          ctx.move_cursor(step as BfOffset);
        }
      }

    }

    ctx.move_cursor(offset);

    true
  }
}

pub struct Context {
  cells: Vec<BfValue>,
  cursor: isize,
  pc: usize
}

impl Context {
  pub fn new(size: usize) -> Self {
    assert!(size <= BfOffset::max_value() as usize, "the buffer is too large");

    Context {
      cells: vec![0; size],
      cursor: 0,
      pc: 0
    }
  }

  #[inline]
  pub fn cell(&self) -> BfValue {
    self.cell_at(0)
  }

  #[inline]
  pub fn cell_mut(&mut self) -> &mut BfValue {
    self.cell_at_mut(0)
  }

  #[inline]
  pub fn cell_at(&self, offset: BfOffset) -> BfValue {
    self.cells[(self.cursor + offset as isize) as usize]
  }

  #[inline]
  pub fn cell_at_mut(&mut self, offset: BfOffset) -> &mut BfValue {
    &mut self.cells[(self.cursor + offset as isize) as usize]
  }

  #[inline]
  pub fn move_cursor(&mut self, amount: BfOffset) {
    self.cursor += amount as isize;
    if self.cursor < 0 {
      println!("cursor {} amount {}", self.cursor, amount);
    }
  }

  #[inline]
  pub fn jump(&mut self, val: BfOffset){
    self.pc = val as usize;
  }

  #[inline]
  pub fn read(&self) -> BfValue {
    stdin().bytes().next().map_or(0, |val| {
      val.unwrap() as i8
    })
  }

  #[inline]
  pub fn write(&self, val: BfValue) {
    print!("{}", val as u8 as char)
  }

  #[inline]
  pub fn execute(&mut self, code: &[Instruction]) {
    let old = self.pc;
    self.pc = 0;
    while code[self.pc].execute(self) {
      self.pc += 1;
    }
    self.pc = old;
  }
}

