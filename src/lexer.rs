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
  Else,
  For,
  Func,
  If,
  Return,
  While,

  // Symbols
  Arr, // ->
  Col, // :
  Com, // ,
  Dot, // .
  Ass, // =

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
  Dol, // $
  Pct, // %
  Car, // ^
  And, // &
  Or,  // |
  Not, // !
  Neg, // ~

  // Comparisons
  Gt,  // >
  Ge,  // >=
  Lt,  // <
  Le,  // <=
  Ne,  // !=
  Eql, // ==
}

use self::Token::*;


pub fn lex(input: &str) {
  let mut tokens: Vec<Token> = Vec::new();
  let mut it = input.chars().peekable();

  while let Some(&c) = it.peek() {
    match c {
      '-' => {
        it.next();
        match it.peek() {
          Some(&'>') => {
            tokens.push(Arr);
            it.next();
          }
          _ => {
            tokens.push(Sub);
          }
        }
      }
      '+' => {tokens.push(Add); it.next();}
      '/' => {tokens.push(Div); it.next();}
      '*' => {tokens.push(Mul); it.next();}
      '(' => {tokens.push(Pal); it.next();}
      ')' => {tokens.push(Par); it.next();}
      '[' => {tokens.push(Sql); it.next();}
      ']' => {tokens.push(Sqr); it.next();}
      '{' => {tokens.push(Cul); it.next();}
      '}' => {tokens.push(Cur); it.next();}
      '<' => {tokens.push(Lt); it.next();}
      '>' => {tokens.push(Gt); it.next();}
      _ => {it.next();}
    }
  }

  println!("{:?}", tokens);
}
