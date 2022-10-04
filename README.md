# typescript

This is a Rust native implementation of a Typescript Parser and a JIT execution engine.

+ [typescript-ast]: Parses Typescript into an AST.
+ [typescript-jit]: Takes an AST representation and creates an executable script.

## Why?

+ Because V8 is to compllicated to "just integrate it".
+ rquickjs makes problems in an multi threaded environment.
+ And lua is fast, but ugly for larger scripts.

## Features

+ Parses Typescript via Pest.
+ A LLVM based JIT execution engine.

## typescript-jit usage

**Info:**
```
A native Typescript parser and JIT runner.

Usage: typescript-jit [OPTIONS] <FILENAME>

Arguments:
  <FILENAME>  

Options:
  -l, --log <LOG>  show a execution log. This for debugging
  -i, --ir <IR>    shows the LLVM IR code. This for debugging
  -h, --help       Print help information
  -V, --version    Print version information
```

```bash
cargo run -- -l typescript.log --ir main.ir 'samples/sample.ts'
```