
use std::num::Wrapping;

use interpreter::*;
use itertools::*;


pub fn optimize<I>(code: I) -> impl Iterator<Item=Instruction>
  where I: Iterator<Item=Instruction> {

  let code = collapse_adds(code);
  let code = collapse_moves(code, false);

  let code = collapse_moves(code, true);

  code
}


pub fn collapse_moves<I>(code: I, compact: bool) -> impl Iterator<Item=Instruction>
  where I: Iterator<Item=Instruction> {

  combine(code, Instruction::from(OpCode::NoOp, 0), move |a, b| {

    match a.opcode {
      OpCode::JumpIfZero | OpCode::JumpIfNonZero => None,
      
      OpCode::NoOp if a.offset == 0 => Some(*b),

      OpCode::NoOp | _ if compact => match b.opcode {
        OpCode::NoOp => Some(Instruction::from(a.opcode, a.offset + b.offset)),
        _ => None
      },

      _ => None
    }
  })
}

pub fn collapse_adds<I>(code: I) -> impl Iterator<Item=Instruction>
  where I: Iterator<Item=Instruction> {
    
  combine(code, Instruction::from_op(OpCode::Add(Wrapping(0))), |a, b| {
    match a.opcode {
      OpCode::Add(n) if a.offset == 0 => match b.opcode {

        OpCode::Add(m) => Some(Instruction::from(OpCode::Add(n+m), b.offset)),

        _ => if n == Wrapping(0) { Some(*b) } else { None }
      }, 

      _ => None
    }
  })
}

pub fn resolve_jumps(code: &mut [Instruction]) -> Result<(), &'static str> {
  let mut stack: Vec<usize> = vec![];

  for i in 0..code.len() {
    match code[i].opcode {
      OpCode::JumpIfZero => {
        stack.push(i);
      },

      OpCode::JumpIfNonZero => {
        let j = match stack.pop() {
          Some(val) => val,
          None => return Err("parse error: dangling ]")
        };

        code[i].offset = j as BfOffset;
        code[j].offset = i as BfOffset;
      },

      _ => () //Do nothing
    }
  }

  return if stack.is_empty() {
    Ok(())
  } else {
    Err("parse error: dangling [")
  }
}