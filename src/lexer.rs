use std;

#[derive(Debug,Clone,PartialEq)]
pub enum Token {
  // Structure
  EOF,
  Enter,
  Exit,
  Space,
  Comment(String),

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


fn lex_pair(next: char, solo: Token, pair: Token, it: &mut std::iter::Peekable<std::str::Chars>) -> Token {
  it.next();

  if let Some(&c) = it.peek() {
    if c == next {
      it.next();
      return pair;
    }
  }

  // is this right? if peek() returns a None, we just return `solo`?
  // seems right. None should be the EOF, meaning `solo` is the last token
  solo
}


pub fn lex(input: &str) -> Vec<Token> {
  let mut tokens: Vec<Token> = Vec::new();
  let mut it = input.chars().peekable();

  while let Some(&c) = it.peek() {
    let x = match c {
      '0'...'9' => lex_number(&mut it),
      '-' => lex_pair('>', Sub, Arr, &mut it),
      '<' => lex_pair('=', Lt, Le, &mut it),
      '>' => lex_pair('=', Gt, Ge, &mut it),
      '=' => lex_pair('=', Ass, Eql, &mut it),
      '!' => lex_pair('=', Not, Ne, &mut it),
      ':' => lex_pair(':', Col, Meta, &mut it),
      '+' => {it.next(); Add}
      '/' => {it.next(); Div}
      '*' => {it.next(); Mul}
      '(' => {it.next(); Pal}
      ')' => {it.next(); Par}
      '[' => {it.next(); Sql}
      ']' => {it.next(); Sqr}
      '{' => {it.next(); Cul}
      '}' => {it.next(); Cur}
      _ => {it.next(); Space}
    };

    match x {
     Space => (),
     Comment(_) => (),
      _ => tokens.push(x),
    }

  }

  tokens
}
