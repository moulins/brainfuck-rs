

use std::io::{stdin, Read};
use std::num::Wrapping;
use std::ops::{Index, IndexMut};

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

    match self.opcode {
      OpCode::Halt => return false,

      OpCode::NoOp => (), //Do nothing

      OpCode::Set(val) => {
        ctx[0] = val;
      },

      OpCode::Add(val) => {
        ctx[0] = ctx[0].wrapping_add(val);
      },

      OpCode::MoveAndAddTo(offset) => {
        if ctx[0] != 0 {
          let offset = offset as BfOffset;
          ctx[offset] = ctx[offset].wrapping_add(ctx[0]);
          ctx[0] = 0;
        }
      }

      OpCode::Input => {
        ctx[0] = ctx.read();
      },

      OpCode::Output => {
        ctx.write(ctx[0]);
      },

      OpCode::JumpIfZero => {
        if ctx[0] == 0 {
          ctx.jump(offset);
        }
        offset = 0;
      },

      OpCode::JumpIfNonZero => {
        if ctx[0] != 0 {
            ctx.jump(offset);
        }
        offset = 0;
      },

      OpCode::FindZero(step) => {
        while ctx[0] != 0 {
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

impl Index<BfOffset> for Context {
  type Output = BfValue;

  #[inline]
  fn index(&self, idx: BfOffset) -> &BfValue {
    let idx = self.get_index(idx);
    //this is ok because get_index panics if out of bounds
    unsafe {
      self.cells.get_unchecked(idx)
    }
  }
}

impl IndexMut<BfOffset> for Context {
  #[inline]
  fn index_mut(&mut self, idx: BfOffset) -> &mut BfValue {
    let idx = self.get_index(idx);
    //this is ok because get_index panics if out of bounds
    unsafe {
      self.cells.get_unchecked_mut(idx)
    }
  }
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
  fn get_index(&self, offset: BfOffset) -> usize {
    let res = (self.cursor + offset as isize) as usize;
    if res >= self.cells.len() {
      panic!("index out of bounds! ({}, len is {}", res as isize, self.cells.len());
    }
    res
  }

  #[inline]
  pub fn move_cursor(&mut self, amount: BfOffset) {
    self.cursor += amount as isize;

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

