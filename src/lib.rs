extern crate bincode;
extern crate blake2;
extern crate codemap;
extern crate serde;

#[macro_use]
extern crate serde_derive;

pub mod code;
pub mod compiler;
pub mod engine;
pub mod lexer;
pub mod module;
pub mod parser;
pub mod semck;

const VERSION: [u8; 4] = [0, 0, 1, 0];
