use std;

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
  Ass, // =
  Col, // :
  Com, // ,
  Dot, // .
  Meta,// ::
  Semi,// ;

  // Braces
  Cul, // {
  Cur, // }
  Pal, // (
  Par, // )
  Sql, // [
  Sqr, // ]

  // Operators
  Add, // +
  And, // &
  Car, // ^
  Div, // /
  Dol, // $
  Mul, // *
  Neg, // ~
  Not, // !
  Or,  // |
  Pct, // %
  Sub, // -

  // Comparisons
  Eql, // ==
  Ge,  // >=
  Gt,  // >
  Le,  // <=
  Lt,  // <
  Ne,  // !=
}

use self::Token::*;


fn lex_number(it: &mut std::iter::Peekable<std::str::Chars>) -> Token {
  let mut digits = String::new();
  while let Some(&c) = it.peek() {
    match c {
      '0'...'9' | '.' => {
        it.next();
        digits.push(c);
      }
      _ => {break}
    }
  }

  match digits.contains(".") {
    true => Float(digits.parse::<f64>().unwrap()),
    false => Int(digits.parse::<i64>().unwrap())
  }
}


pub fn lex(input: &str) {
  let mut tokens: Vec<Token> = Vec::new();
  let mut it = input.chars().peekable();

  while let Some(&c) = it.peek() {
    match c {
      '0'...'9' => {
        tokens.push(lex_number(&mut it));
      }
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
