
use interpreter::*;
use std::num::Wrapping;
use std::collections::VecDeque;
use itertools::*;

pub struct Pattern<'a>(
  &'a [OpCode],
  &'a (Fn(&VecDeque<Instruction>) -> Option<Instruction> + Sync)
);


// [+] or [-] sets the current cell to zero
pub static SET_ZERO_LOOP: Pattern = Pattern(&[
    OpCode::JumpIfZero,
    OpCode::Add(0),
    OpCode::JumpIfNonZero,
  ],
  &|list| match list[1].opcode {
      OpCode::Add(a) if a.abs() == 1 => {
        Some(Instruction::from_op(OpCode::Set(0)))
      },
      _ => None
  });

pub fn replace<'a, I>(code: I, pat: &'a Pattern) -> impl Iterator<Item=Instruction> + 'a
  where I: Iterator<Item=Instruction> + 'a {

  use std::mem::*;

  let Pattern(ref template, ref matcher) = *pat;
  replace_template(
    code,
    template,
    |a, b| a.offset == 0 && discriminant(&a.opcode) == discriminant(b),
    matcher
  )
}
