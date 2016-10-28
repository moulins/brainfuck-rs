
use std::num::Wrapping;

use interpreter::*;
use itertools::*;
use patterns;

pub fn optimize<'a, I: 'a>(code: I) -> impl Iterator<Item=Instruction> + 'a
  where I: Iterator<Item=Instruction> {

  let code = collapse_adds(code);
  let code = collapse_moves(code, false);

  let code = pattern_set_cell(code);
  let code = pattern_seek_zero(code);
  let code = pattern_move_cell(code);

  let code = collapse_moves(code, true);

  code
}

pub fn pattern_set_cell<I>(code: I) -> impl Iterator<Item=Instruction>
  where I: Iterator<Item=Instruction> {
  // [+] or [-] sets the current cell to zero
  let code = fixed_pattern!(
    _ = OpCode::JumpIfZero,
    _ = OpCode::Add(incr),
    _ = OpCode::JumpIfNonZero
    => {
      if incr.abs() == 1 {
        Some(Instruction::from_op(OpCode::Set(0)))
      } else { None }
    }
  )(code);

  fixed_pattern!(
    _ = OpCode::Set(val),
    _ = OpCode::Add(incr)
    => {
      let res = val.wrapping_add(incr);
      Some(Instruction::from_op(OpCode::Set(res)))
    }
  )(code)

}

pub fn pattern_seek_zero<I>(code: I) -> impl Iterator<Item=Instruction>
  where I: Iterator<Item=Instruction> {
  //[<] or [>] seeks a zero cell
  fixed_pattern!(
    _ = OpCode::JumpIfZero,
    m = OpCode::NoOp,
    _ = OpCode::JumpIfNonZero
    => {
      //if the offset can be represented as a BfValue
      m.offset_to_value().map(|o| {
        Instruction::from_op(OpCode::FindZero(o))
      })
    }
  )(code)
}

pub fn pattern_move_cell<I>(code: I) -> impl Iterator<Item=Instruction>
  where I: Iterator<Item=Instruction> {
  //[->>+<<] moves a value
  fixed_pattern!(
    _    = OpCode::JumpIfZero,
    _    = OpCode::Add(decr),
    to   = OpCode::NoOp,
    _    = OpCode::Add(incr),
    from = OpCode::NoOp, 
    _    = OpCode::JumpIfNonZero
    => {
      if !(incr.abs() == 1 && decr == -incr)
      || !(to.offset == -from.offset) {
        return None;
      }

      to.offset_to_value().map(|o| {
        Instruction::from_op(OpCode::MoveAndAddTo(o))
      })
    }  
  )(code)
}

pub fn collapse_moves<I>(code: I, compact: bool) -> impl Iterator<Item=Instruction>
  where I: Iterator<Item=Instruction> {
  combine(code, Instruction::from(OpCode::NoOp, 0), move |a, b| {

    match (a.opcode, b.opcode) {      
      (OpCode::NoOp, _) if a.offset == 0 => Some(*b),

      (OpCode::NoOp, OpCode::NoOp) => {
        Some(Instruction::from(a.opcode, a.offset + b.offset))
      },

      (_, OpCode::NoOp) if compact && !a.use_offset() => {
        Some(Instruction::from(a.opcode, a.offset + b.offset))
      },

      _ => None
    }
  })
}

pub fn collapse_adds<I>(code: I) -> impl Iterator<Item=Instruction>
  where I: Iterator<Item=Instruction> {
    
  combine(code, Instruction::from_op(OpCode::Add(0)), |a, b| {
    match (a.opcode, b.opcode) {
      (OpCode::Add(0), _) if a.offset == 0 => Some(*b),

      (OpCode::Add(n), OpCode::Add(m)) if a.offset == 0 => {
        Some(Instruction::from(OpCode::Add(n.wrapping_add(m)), b.offset))
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