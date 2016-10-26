
use interpreter::*;
use std::num::Wrapping;
use std::collections::VecDeque;
use itertools::*;

pub struct Pattern<'a>(
  &'a [OpCode],
  &'a (Fn(&VecDeque<Instruction>) -> Option<Instruction> + Sync)
);


const ZERO  : BfValue = Wrapping(0);
const ONE   : BfValue = Wrapping(1);
const M_ONE : BfValue = Wrapping(-1);

pub static SET_ZERO_LOOP: Pattern = Pattern(&[
    OpCode::JumpIfZero,
    OpCode::Add(ZERO),
    OpCode::JumpIfNonZero,
  ],
  &|list| match list[1].opcode {
      OpCode::Add(ONE) | OpCode::Add(M_ONE) => {
        Some(Instruction::from_op(OpCode::Set(ZERO)))
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
    |a, b| discriminant(&a.opcode) == discriminant(b),
    matcher
  )
}
