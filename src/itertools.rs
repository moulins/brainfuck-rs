
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
