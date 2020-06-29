#![feature(llvm_asm, integer_atomics)]

extern crate serde;
extern crate serde_aux;
#[macro_use]
extern crate serde_derive;
extern crate toml;

/// This module is used for parsing the client configuration file.
pub mod config;

/// This module contains the CPU cycles related functionality; rdtsc() etc.
pub mod cycles;
