# typescript-jit

This is a Rust native implementation of a Typescript Parser and a JIT execution engine.

## Why?

+ Because V8 is to compllicated to "just integrate it".
+ rquickjs makes problems in an multi threaded environment.
+ And lua is fast, but ugly for larger scripts.

## Features

+ Parses Typescript via Pest.
+ A LLVM based JIT execution engine.