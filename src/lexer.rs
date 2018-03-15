#[derive(Debug,Clone,PartialEq)]
pub enum Token {
  // Structure
  EOF,
  Enter,
  Exit,

  // Literals
  Null,
  Bool(bool),
  Float(f64),
  Int(i64),
  StrLit(String),
  Name(String),

  // Keywords
  Break,
  Continue,
  Func,
  If,
  Return,

  // Symbols
  Arr, // ->
  Col, // :
  Com, // ,
  Dot, // .
  Eql, // =

  // Braces
  Cul, // {
  Cur, // }
  Pal, // (
  Par, // )
  Sql, // [
  Sqr, // ]

  // Operators
  Add, // +
  Div, // /
  Mul, // *
  Sub, // -
}

use self::Token::*;


pub fn lex(path: &str) {
  println!("HELLO I AM LEXING {}", path);
}
