
use interpreter::*;
use std::num::Wrapping;
use std::collections::VecDeque;
use itertools::*;

macro_rules! fixed_pattern {
  (@replace $_t:tt $sub:expr) => {$sub};
  (@count_tts $($tts:tt)*) => {
    0usize $(+ fixed_pattern!(@replace $tts 1usize))*
  };
    
  (@declare_vars {
    iter: $iter:ident,
    names: [$name:pat, $($names_t:tt),*],
    patterns: [$pat:pat, $($pats_t:tt),*],
    expr: $expr:expr
  }) => {{
    let _cur = $iter.next();
    if let Some(&$crate::interpreter::Instruction { opcode: $pat, .. }) = _cur {
      let _cur = _cur.unwrap();
      if !_cur.use_offset() && _cur.offset != 0 {
        return None;
      }
      let $name = _cur;
      fixed_pattern!(@declare_vars {
        iter:$iter,
        names: [$($names_t),*],
        patterns: [$($pats_t),*],
        expr: $expr
      })
    } else { None }
  }};
    
  (@declare_vars {
    iter: $iter:ident,
    names: [END],
    patterns: [END],
    expr: $expr:expr
  }) => {$expr};
    
  ($($names:pat = $pats:pat),* => $expr:expr) => { |code| {

    let pat_size = fixed_pattern!(@count_tts $($names)*);

    $crate::itertools::replace_sized_pattern(code, pat_size,
      |_list: &::std::collections::VecDeque<$crate::interpreter::Instruction>| {
      let mut _iter = _list.iter();
      fixed_pattern!(@declare_vars {
        iter: _iter,
        names: [$($names),*, END],
        patterns: [$($pats),*, END],
        expr: $expr
      })
    }) 
  }};
}



