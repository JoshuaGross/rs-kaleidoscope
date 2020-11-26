# Kaleidoscope / Rust

This is an implementation of a Kaleidoscope-ish language-to-LLVM-IR JIT compiler in Rust.

This is not a true implementation of Kaleidoscope because I didn't want to implement whitespace-sensitive parsing just yet.

One of my goals was to *not* use Nom's macros, which results in the code being a little more verbose than it would otherwise be.

Installation:

1. Use [`llvmenv`](https://github.com/termoshtt/llvmenv) to install llvm 10.0
