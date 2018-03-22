use codemap::File;
use codemap::Spanned;
use self::Token::*;
use std::iter::Enumerate;
use std::iter::Peekable;
use std::str::Chars;

type LexIter<'a> = Peekable<Enumerate<Chars<'a>>>;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  // Structure
  EOF,
  Enter,
  Exit,
  Space,
  End,
  Comment(String),

  // Literals
  Null,
  Bool(bool),
  Float(f64),
  Int(i64),
  Str(String),
  Name(String),

  // Keywords
  Break,
  Catch,
  Continue,
  Else,
  For,
  Func,
  If,
  Import,
  In,
  Loop,
  Pass,
  Return,
  Save,
  Var,
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
  At,  // @
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


fn lex_number(it: &mut LexIter) -> Token {
  let mut digits = String::new();
  while let Some(&(_i, c)) = it.peek() {
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

  while let Some(&(_i, c)) = it.peek() {
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
    "null" => Null,

    "break" => Break,
    "catch" => Catch,
    "continue" => Continue,
    "else" => Else,
    "for" => For,
    "func" => Func,
    "if" => If,
    "import" => Import,
    "in" => In,
    "loop" => Loop,
    "pass" => Pass,
    "return" => Return,
    "save" => Save,
    "var" => Var,
    "while" => While,

    _ => Name(name)
  }
}


fn lex_comment(it: &mut LexIter) -> Token {
  let mut comment = String::new();
  it.next();

  while let Some(&(_i, c)) = it.peek() {
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

  while let Some(&(_i, c)) = it.peek() {
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

  if let Some(&(_i, c)) = it.peek() {
    if c == next {
      it.next();
      return pair;
    }
  }

  // is this right? if peek() returns a None, we just return `solo`?
  // seems right. None should be the EOF, meaning `solo` is the last token
  solo
}


pub fn lex(input: &File) -> Vec<Spanned<Token>> {
  let mut tokens: Vec<Spanned<Token>> = Vec::new();
  let mut it: LexIter = input.source().chars().enumerate().peekable();
  let mut indent_stack: Vec<u64> = Vec::new();
  let mut current_indent: u64 = 0;

  // start at indentation 0
  indent_stack.push(current_indent);

  while let Some(&(i, c)) = it.peek() {
    let x = if current_indent < indent_stack[indent_stack.len() - 1] {
      indent_stack.pop();
      Exit
    }
    else {
      match c {
        '#' => lex_comment(&mut it),
        'a'...'z' | 'A'...'Z' | '_' => lex_name(&mut it),
        '0'...'9' => lex_number(&mut it),
        '\n' => {
          let indent = lex_indent(&mut it);

          if let Some(&(_, c)) = it.peek() {
            match c {
              '\n' => Space,
              _ => {
                current_indent = indent;
                // if this panics, there's a bug - indent_stack should always have a 0
                if indent > indent_stack[indent_stack.len() - 1] {
                  indent_stack.push(indent);
                  Enter
                }
                else {
                  End
                }
              }
            }
          }
          else {
            End
          }
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
        '@' => {it.next(); At}
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
      }
    };

    // figure out what the span was for this token
    // either there's something we can peek, or the span is until EOF
    let mut end_i = if let Some(&(j, _)) = it.peek() {
      j
    }
    else {
      input.source().len()
    };
    let span = input.span.subspan(i as u64, end_i as u64);

    match x {
      // don't emit tokens for spaces or comments
      Space => (),
      Comment(_) => (),

      // don't insert duplicate newlines, or file-leading newlines
      End => {
        match tokens.last().cloned() {
          None => (),
          Some(x) => {
            if x.node != End {
              tokens.push(Spanned {node: End, span: span});
            }
          },
        }
      }

      // exit should always be followed by a End
      Exit => {
        tokens.push(Spanned {node: Exit, span: span});
        tokens.push(Spanned {node: End, span: span});
      }

      // emit everything else
      _ => tokens.push(Spanned {node: x, span: span}),
    }

  }

  // make a span for all closing tokens
  let end = input.source().len() as u64;
  let span = input.span.subspan(end, end);

  // exit blocks that are open at EOF
  while indent_stack.len() > 1 {
    tokens.push(Spanned {node: Exit, span: span});
    tokens.push(Spanned {node: End, span: span});
    indent_stack.pop();
  }

  // push the EOF token
  tokens.push(Spanned {node: EOF, span: span});

  tokens
}

#[cfg(test)]
mod tests {
  use super::*;
  use codemap::CodeMap;

  #[test]
  fn lex_numbers() {
    let mut map = CodeMap::new();
    let file = map.add_file(String::from("_test"), String::from("0 5 05 5.3 1.234 1. 0. 0.0"));
    let tokens = lex(&file);
    assert_eq!(tokens.len(), 9);
    assert_eq!(tokens[0].node, Int(0));
    assert_eq!(tokens[1].node, Int(5));
    assert_eq!(tokens[2].node, Int(5));
    assert_eq!(tokens[3].node, Float(5.3));
    assert_eq!(tokens[4].node, Float(1.234));
    assert_eq!(tokens[5].node, Float(1.0));
    assert_eq!(tokens[6].node, Float(0.0));
    assert_eq!(tokens[7].node, Float(0.0));
    assert_eq!(tokens[8].node, EOF);
  }

  #[test]
  fn lex_keywords() {
    let mut map = CodeMap::new();
    let file = map.add_file(String::from("_test"), String::from("break catch continue else for func if import in loop pass return save var while name true false null"));
    let tokens = lex(&file);
    assert_eq!(tokens.len(), 20);
    assert_eq!(tokens[0].node, Break);
    assert_eq!(tokens[1].node, Catch);
    assert_eq!(tokens[2].node, Continue);
    assert_eq!(tokens[3].node, Else);
    assert_eq!(tokens[4].node, For);
    assert_eq!(tokens[5].node, Func);
    assert_eq!(tokens[6].node, If);
    assert_eq!(tokens[7].node, Import);
    assert_eq!(tokens[8].node, In);
    assert_eq!(tokens[9].node, Loop);
    assert_eq!(tokens[10].node, Pass);
    assert_eq!(tokens[11].node, Return);
    assert_eq!(tokens[12].node, Save);
    assert_eq!(tokens[13].node, Var);
    assert_eq!(tokens[14].node, While);
    assert_eq!(tokens[15].node, Name(String::from("name")));
    assert_eq!(tokens[16].node, Bool(true));
    assert_eq!(tokens[17].node, Bool(false));
    assert_eq!(tokens[18].node, Null);
    assert_eq!(tokens[19].node, EOF);
  }

  #[test]
  fn lex_symbols() {
    let mut map = CodeMap::new();
    let file = map.add_file(String::from("_test"), String::from("-> = : : , . :: ; {} () [] +&@^/$*~!|%- == >= - > = <= < = != ! ="));
    let tokens = lex(&file);
    assert_eq!(tokens.len(), 38);
    assert_eq!(tokens[0].node, Arr);
    assert_eq!(tokens[1].node, Ass);
    assert_eq!(tokens[2].node, Col);
    assert_eq!(tokens[3].node, Col);
    assert_eq!(tokens[4].node, Com);
    assert_eq!(tokens[5].node, Dot);
    assert_eq!(tokens[6].node, Meta);
    assert_eq!(tokens[7].node, Semi);

    assert_eq!(tokens[8].node, Cul);
    assert_eq!(tokens[9].node, Cur);
    assert_eq!(tokens[10].node, Pal);
    assert_eq!(tokens[11].node, Par);
    assert_eq!(tokens[12].node, Sql);
    assert_eq!(tokens[13].node, Sqr);

    assert_eq!(tokens[14].node, Add);
    assert_eq!(tokens[15].node, And);
    assert_eq!(tokens[16].node, At);
    assert_eq!(tokens[17].node, Car);
    assert_eq!(tokens[18].node, Div);
    assert_eq!(tokens[19].node, Dol);
    assert_eq!(tokens[20].node, Mul);
    assert_eq!(tokens[21].node, Neg);
    assert_eq!(tokens[22].node, Not);
    assert_eq!(tokens[23].node, Or);
    assert_eq!(tokens[24].node, Pct);
    assert_eq!(tokens[25].node, Sub);
    assert_eq!(tokens[26].node, Eql);
    assert_eq!(tokens[27].node, Ge);
    assert_eq!(tokens[28].node, Sub);
    assert_eq!(tokens[29].node, Gt);
    assert_eq!(tokens[30].node, Ass);
    assert_eq!(tokens[31].node, Le);
    assert_eq!(tokens[32].node, Lt);
    assert_eq!(tokens[33].node, Ass);
    assert_eq!(tokens[34].node, Ne);
    assert_eq!(tokens[35].node, Not);
    assert_eq!(tokens[36].node, Ass);
    assert_eq!(tokens[37].node, EOF);
  }

  // TODO add lex_structure for testing enter/exit/comment/etc
}
