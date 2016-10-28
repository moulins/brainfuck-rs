# brainfuck-rs
Yet Another Brainfuck Interpreter written in Rust, because why the f$%* not? (And it's a good way to learn a new language)  
If you don't know brainfuck yet, go [here](https://esolangs.org/wiki/brainfuck).

Compile with the `--release` for better performance.


## Implementation

For now, the interpreter supports only a 30,000-cell memory, and will panic when a brainfuck program tries to access cells outside this range.

The cells are 8 bits wide and are wrapping, so that `255 + 1 = 0`.

Executing the `,` instruction on `EOF` will set the current cell to `0`.
