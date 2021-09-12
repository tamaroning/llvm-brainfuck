# LLVM-brainfuck
Brainfuck compiler using LLVM as a backend.  
LLVM-brainfuck uses inkwell, a LLVM wrapper written in Rust.

# Usage

## Build

```sh
git clone https://github.com/tamaroning/llvm-brainfuck
cd llvm-brainfuck
cargo build --release
```

## JIT Compile and Run
```sh
target/release/llvm-brainfuck [source code]
```

# Example
```sh
$ target/release/llvm-brainfuck "+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.+.+.>++++++++++."
ABC
$ 
```

# Feature
```+, -, >```is already implemented.
