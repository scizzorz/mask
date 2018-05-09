extern crate bincode;
extern crate blake2;
extern crate codemap;
extern crate gc;
extern crate ordered_float;
extern crate serde;

#[macro_use]
extern crate gc_derive;

#[macro_use]
extern crate serde_derive;

pub mod code;
pub mod compiler;
pub mod data;
pub mod engine;
pub mod lexer;
pub mod module;
pub mod parser;
pub mod semck;

const VERSION: [u8; 4] = [0, 0, 1, 0];

type float_base = f64;
type float = ordered_float::OrderedFloat<float_base>;
type int = i64;

// ordered_float doesn't implement these, so we need to manually derive here
#[derive(Serialize, Deserialize)]
#[serde(remote = "ordered_float::OrderedFloat::<f64>")]
struct FloatDef(f64);
