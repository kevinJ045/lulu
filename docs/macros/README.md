# Introduction to Macros

Lulu's macro system is one of its most powerful features. Inspired by Rust(Obviously), it allows you to perform compile-time code generation and transformation, effectively extending Lua with new syntax and features.

## How Macros Work

Macros are special functions that run at compile time. They take code as input (in the form of tokens) and produce new code as output. This new code is then added into your final bundle.

This means you can create abstractions that have **zero runtime cost**. The logic you write in macros is executed and resolved before your Lua code ever runs.

Macros are identified by a `!` at the end of their name, for example `cfg!` or `class!`.

## Macro Types

There are two fundamental types of macros in Lulu:

- **Transforming Macros**: These macros transform the code you provide. A good example is `cfg!`, which conditionally keeps or removes a block of code based on a condition. The output is still your code, just a subset of it.

- **Generating Macros**: These macros generate entirely new code. For example, the `class!` macro takes a high-level class definition and generates all the complex Lua metatable boilerplate required to make it work.

You can read more about specific macros in the files below.