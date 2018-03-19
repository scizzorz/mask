use std;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  // Structure
  EOF,
  Enter,
  Exit,
  Space,
  Newline,
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
  In,
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

type LexIter<'a> = std::iter::Peekable<std::iter::Enumerate<std::str::Chars<'a>>>;


fn lex_number(it: &mut LexIter) -> Token {
  let mut digits = String::new();
  while let Some(&(i, c)) = it.peek() {
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


fn lex_name(it: &mut LexIter) -> Token {
  let mut name = String::new();
  name.push(it.next().unwrap().1);

  while let Some(&(i, c)) = it.peek() {
    match c {
      'a'...'z' | 'A'...'Z' | '0'...'9' | '_' => {
        it.next();
        name.push(c);
      }
      _ => {break}
    }
  }

  match name.as_str() {
    "true" => Bool(true),
    "false" => Bool(false),
    "break" => Break,
    "continue" => Continue,
    "else" => Else,
    "for" => For,
    "func" => Func,
    "if" => If,
    "in" => In,
    "return" => Return,
    "while" => While,
    _ => Name(name)
  }
}


fn lex_comment(it: &mut LexIter) -> Token {
  let mut comment = String::new();
  it.next();

  while let Some(&(i, c)) = it.peek() {
    match c {
      '\n' => {break}
      _ => {
        it.next();
        comment.push(c);
      }
    }
  }

  println!("Comment: {:?}", comment);
  Comment(comment)
}


fn lex_indent(it: &mut LexIter) -> u64 {
  let mut indent: u64 = 0;
  it.next();

  while let Some(&(i, c)) = it.peek() {
    match c {
      ' ' => {
        it.next();
        indent += 1;
      }
      _ => {break}
    }
  }

  indent
}


fn lex_pair(next: char, solo: Token, pair: Token, it: &mut LexIter) -> Token {
  it.next();

  if let Some(&(i, c)) = it.peek() {
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
  let mut it: LexIter = input.chars().enumerate().peekable();

  while let Some(&(i, c)) = it.peek() {
    let start_i = i;
    let x = match c {
      '#' => lex_comment(&mut it),
      'a'...'z' | 'A'...'Z' | '_' => lex_name(&mut it),
      '0'...'9' => lex_number(&mut it),
      '\n' => {
        lex_indent(&mut it);
        Newline
      }

      // Compound
      '-' => lex_pair('>', Sub, Arr, &mut it),
      '<' => lex_pair('=', Lt, Le, &mut it),
      '>' => lex_pair('=', Gt, Ge, &mut it),
      '=' => lex_pair('=', Ass, Eql, &mut it),
      '!' => lex_pair('=', Not, Ne, &mut it),
      ':' => lex_pair(':', Col, Meta, &mut it),

      // Symbols
      // -> Arr
      // = Ass
      // : Col
      ',' => {it.next(); Com}
      '.' => {it.next(); Dot}
      // :: Meta
      ';' => {it.next(); Semi}

      // Braces
      '(' => {it.next(); Pal}
      ')' => {it.next(); Par}
      '[' => {it.next(); Sql}
      ']' => {it.next(); Sqr}
      '{' => {it.next(); Cul}
      '}' => {it.next(); Cur}

      // Operators
      '+' => {it.next(); Add}
      '&' => {it.next(); And}
      '^' => {it.next(); Car}
      '/' => {it.next(); Div}
      '$' => {it.next(); Dol}
      '*' => {it.next(); Mul}
      '~' => {it.next(); Neg}
      // ! Not
      '|' => {it.next(); Or}
      '%' => {it.next(); Pct}
      // - Sub

      _ => {it.next(); Space}
    };

    // don't emit tokens for spaces or comments
    match x {
      Space => (),
      Comment(_) => (),
      _ => tokens.push(x),
    }

    let mut end_i = 0;
    if let Some(&(j, _)) = it.peek() {
      end_i = j;
      println!("tokenized from {:?} to {:?}", start_i, end_i);
    }
    else {
      println!("tokenized from {:?} to end", start_i);
    }
  }

  tokens.push(Newline);
  tokens.push(EOF);

  tokens
}
