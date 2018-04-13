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
  Tab, // lol
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
  Table,
  Var,
  While,

  // Symbols
  Arr,  // ->
  Ass,  // =
  Col,  // :
  Com,  // ,
  Dot,  // .
  Meta, // ::
  Semi, // ;

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
      _ => break,
    }
  }

  match digits.contains(".") {
    true => Float(digits.parse::<f64>().unwrap()),
    false => Int(digits.parse::<i64>().unwrap()),
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
      _ => break,
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
    "fn" => Func,
    "if" => If,
    "import" => Import,
    "in" => In,
    "loop" => Loop,
    "pass" => Pass,
    "return" => Return,
    "save" => Save,
    "table" => Table,
    "var" => Var,
    "while" => While,

    _ => Name(name),
  }
}

fn lex_comment(it: &mut LexIter) -> Token {
  let mut comment = String::new();
  it.next();

  while let Some(&(_i, c)) = it.peek() {
    match c {
      '\n' => break,
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
      _ => break,
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
    } else {
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
                } else {
                  End
                }
              }
            }
          } else {
            End
          }
        }
        '\t' => {
          it.next();
          Tab
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
        ',' => {
          it.next();
          Com
        }
        '.' => {
          it.next();
          Dot
        }
        // :: Meta
        ';' => {
          it.next();
          Semi
        }

        // Braces
        '(' => {
          it.next();
          Pal
        }
        ')' => {
          it.next();
          Par
        }
        '[' => {
          it.next();
          Sql
        }
        ']' => {
          it.next();
          Sqr
        }
        '{' => {
          it.next();
          Cul
        }
        '}' => {
          it.next();
          Cur
        }

        // Operators
        '+' => {
          it.next();
          Add
        }
        '&' => {
          it.next();
          And
        }
        '@' => {
          it.next();
          At
        }
        '^' => {
          it.next();
          Car
        }
        '/' => {
          it.next();
          Div
        }
        '$' => {
          it.next();
          Dol
        }
        '*' => {
          it.next();
          Mul
        }
        '~' => {
          it.next();
          Neg
        }
        // ! Not
        '|' => {
          it.next();
          Or
        }
        '%' => {
          it.next();
          Pct
        }
        // - Sub
        _ => {
          it.next();
          Space
        }
      }
    };

    // figure out what the span was for this token
    // either there's something we can peek, or the span is until EOF
    let mut end_i = if let Some(&(j, _)) = it.peek() {
      j
    } else {
      input.source().len()
    };
    let span = input.span.subspan(i as u64, end_i as u64);

    match x {
      // don't emit tokens for spaces or comments
      Space => (),
      Comment(_) => (),

      // don't insert duplicate newlines, or file-leading newlines
      End => match tokens.last().cloned() {
        None => (),
        Some(x) => {
          if x.node != End {
            tokens.push(Spanned {
              node: End,
              span: span,
            });
          }
        }
      },

      // exit should always be followed by a End
      Exit => {
        tokens.push(Spanned {
          node: Exit,
          span: span,
        });
        tokens.push(Spanned {
          node: End,
          span: span,
        });
      }

      // emit everything else
      _ => tokens.push(Spanned {
        node: x,
        span: span,
      }),
    }
  }

  // make a span for all closing tokens
  let end = input.source().len() as u64;
  let span = input.span.subspan(end, end);

  // sometimes a trailing newline goes missing before EOF
  if let Some(x) = tokens.last().cloned() {
    if x.node != End {
      tokens.push(Spanned {
        node: End,
        span: span,
      });
    }
  }

  // exit blocks that are open at EOF
  while indent_stack.len() > 1 {
    tokens.push(Spanned {
      node: Exit,
      span: span,
    });
    tokens.push(Spanned {
      node: End,
      span: span,
    });
    indent_stack.pop();
  }

  // push the EOF token
  tokens.push(Spanned {
    node: EOF,
    span: span,
  });

  tokens
}

#[cfg(test)]
#[path = "./tests/lexer.rs"]
mod tests;
