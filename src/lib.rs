extern crate bincode;
extern crate blake2;
extern crate codemap;
extern crate gc;
extern crate ordered_float;
extern crate serde;
extern crate serde_yaml;

#[macro_use]
extern crate gc_derive;

#[macro_use]
extern crate serde_derive;

pub mod code;
pub mod compiler;
pub mod core;
pub mod data;
pub mod engine;
pub mod lexer;
pub mod module;
pub mod parser;
pub mod prelude;
pub mod semck;

const VERSION: [u8; 4] = [0, 0, 1, 0];

type FloatBase = f64;
type float = ordered_float::OrderedFloat<FloatBase>;
type int = i64;

// ordered_float doesn't implement these, so we need to manually derive here
#[derive(Serialize, Deserialize)]
#[serde(remote = "ordered_float::OrderedFloat::<f64>")]
struct FloatDef(f64);
