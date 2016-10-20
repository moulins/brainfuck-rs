use interpreter::*;
use std::num::Wrapping;

pub fn stateful<I, St, F, L, B>(iter: I, init_state: St, map: F, last: L) -> impl Iterator<Item=B>
  where I: Iterator,
        F: FnMut(St, I::Item) -> (St, B),
        L: FnOnce(St) -> Option<B> {
    
  struct StatefulIter<I, St, F, L> {
    iter: I,
    state: Option<(St, L)>,
    map: F
  }

  impl<I, St, F, L, B> Iterator for StatefulIter<I, St, F, L>
    where I: Iterator,
          F: FnMut(St, I::Item) -> (St, B),
          L: FnOnce(St) -> Option<B> {

    type Item = B;

    fn next(&mut self) -> Option<B> {
      use std::mem::*;

      match self.iter.next() {
        Some(item) => {
          let (state, last) = match replace(&mut self.state, None) {
            Some(s) => s,
            None => return None
          };

          let (state, item) = (self.map)(state, item);
          replace(&mut self.state, Some((state, last)));
          Some(item)
        },

        None => replace(&mut self.state, None)
              .and_then(|state| (state.1)(state.0))
      }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
      let (min, max) = self.iter.size_hint();
      (min, max.and_then(|x| {
        if x == usize::max_value() {
          None
        } else {
          Some(x+1)
        }
      }))
    }
  }

  StatefulIter {
    iter: iter,
    state: Some((init_state, last)),
    map: map
  }
}

pub fn combine<I, F>(iter: I, neutral_value: I::Item, mut combinator: F) -> impl Iterator<Item=I::Item>
  where I: Iterator,
        F: FnMut(&I::Item, &I::Item) -> Option<I::Item> {

  stateful(iter, neutral_value, move |last, cur| {
    match combinator(&last, &cur) {
      Some(res) => (res, None),
      None => (cur, Some(last))
    }
  }, move |last| Some(Some(last))).filter_map(|i| i)
}

pub fn collapse_move<I>(code: I, compact: bool) -> impl Iterator<Item=Instruction>
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

pub fn collapse_add<I>(code: I) -> impl Iterator<Item=Instruction>
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