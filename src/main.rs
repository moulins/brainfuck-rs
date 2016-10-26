#![feature(conservative_impl_trait)]
#![feature(discriminant_value)]
#![feature(static_in_const)]

#![allow(dead_code)]
#![allow(unused_imports)]

mod interpreter;
mod itertools;
mod optimizer;
mod patterns;

use std::fs::File;
use std::io::Read;

use interpreter::*;

use itertools::*;

fn main() {
  let args: Vec<_> = std::env::args().collect();
  if args.len() > 2 {
    panic!("Too much arguments!");
  } else if args.len() < 2 {
    panic!("You must specify a brainfuck file.");
  }

  let file = File::open(&args[1]).unwrap();
  let original_len = file.metadata().unwrap().len();

  let code = process_code(get_bytes_iter(file)
              .map(|b| Instruction::from_char(b as char))
            ).unwrap();

  println!("code length: {} raw, {} optimized", original_len, code.len());

  Context::new(30000).execute(&code);
}

fn process_code<I: Iterator<Item=Instruction>>(code: I) -> Result<Vec<Instruction>, &'static str> {

  let mut code: Vec<_> = optimizer::optimize(code).collect();

  code.push(Instruction::from_op(OpCode::Halt));
  try!(optimizer::resolve_jumps(&mut code));

  Ok(code) 
}

fn get_bytes_iter<T: Read>(reader: T) -> impl Iterator<Item=u8> {
  reader.bytes()
   .map(|r| match r {
    Ok(b) => b,
    Err(e) => panic!(e)
  })
}
