
use std::collections::VecDeque;

pub fn stateful<I, St, F, L, B>(iter: I, init_state: St, map: F, last: L) -> impl Iterator<Item=B>
  where I: Iterator,
        F: FnMut(St, I::Item) -> (St, B),
        L: FnOnce(St) -> Option<B> {
    
  struct StatefulIter<I, St, F, L> {
    state: Option<(St, L)>,
    map: F,
    iter: I
  }

  impl<I, St, F, L, B> Iterator for StatefulIter<I, St, F, L>
    where I: Iterator,
          F: FnMut(St, I::Item) -> (St, B),
          L: FnOnce(St) -> Option<B> {

    type Item = B;

    fn next(&mut self) -> Option<Self::Item> {
      use std::mem::*;

      match self.iter.next() {
        Some(item) => {
          let (state, last) = match self.state.take() {
            Some(s) => s,
            None => return None
          };

          let (state, item) = (self.map)(state, item);
          replace(&mut self.state, Some((state, last)));
          Some(item)
        },

        None => self.state.take().and_then(|state| (state.1)(state.0))
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


pub fn replace_sized_pattern<I, M>(iter: I, pattern_size: usize, matcher: M)
  -> impl Iterator<Item=I::Item>
  where I: Iterator,
        M: FnMut(&VecDeque<I::Item>) -> Option<I::Item> {

  struct PatternIter<I: Iterator, M> {
    iter: I,
    matcher: M,
    size: usize,
    list: VecDeque<I::Item>,
    is_done: bool
  }

  impl<I, M> Iterator for PatternIter<I, M>
    where I: Iterator,
          M: FnMut(&VecDeque<I::Item>) -> Option<I::Item> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
      if !self.is_done {

        for i in &mut self.iter {
          self.list.push_back(i);
          if self.list.len() == self.size {
            return if let Some(res) = (self.matcher)(&self.list) {
              self.list.drain(..);
              Some(res)
            
            } else {
              self.list.pop_front()
            }
          }
        }
      }

      let r = self.list.pop_front();
      if r.is_none() { self.is_done = false }
      r
    }
  }

  assert!(pattern_size > 0);
  PatternIter {
    iter: iter,
    matcher: matcher,
    size: pattern_size,
    list: VecDeque::with_capacity(pattern_size),
    is_done: false
  }
}

/*pub fn replace_template<'a, I, T, E, M>(iter: I, template: &'a [T], equals: E, matcher: M)
  -> impl Iterator<Item=I::Item> + 'a
  where I: Iterator + 'a,
        E: Fn(&I::Item, &T) -> bool + 'a,
        M: Fn(&VecDeque<I::Item>) -> Option<I::Item> + 'a{

  replace_sized_pattern(iter, template.len(), move |list| {
    let is_match = list.iter()
        .zip(template)
        .skip_while(|t| {
          let &(a, b) = t;
              equals(a, b)
        }).next().is_none();

    if is_match {
      matcher(list)
    } else {
      None
    }
  })
}*/
 