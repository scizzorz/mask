use codemap::Spanned;
use lexer::Token;
use std::iter::Peekable;
use std::slice::Iter;

type ParseIter<'a> = Peekable<Iter<'a, Spanned<Token>>>;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
  Block(Vec<Node>),
  If{cond: Box<Node>},
  Stmt(Box<Node>),
  Break,
  Continue,
  Expr,
  Pass,

  // Literals
  Null,
  Bool(bool),
  Float(f64),
  Int(i64),
  Str(String),
  Name(String),

  None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
  UnexpectedToken(Token),
  UnexpectedEOF,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Parse {
  Ok(Node),
  Err(ParseErrorKind),
  None
}

impl Parse {
  fn expect(self, msg: &str) -> Node {
    match self {
      Parse::Ok(n) => n,
      Parse::Err(x) => panic!("{:?} from {:?}", msg, x),
      Parse::None => panic!("{:?}", msg),
    }
  }

  fn map<F>(self, op: F) -> Parse
      where F: FnOnce(Node) -> Node {
    if let Parse::Ok(n) = self {
      Parse::Ok(op(n))
    }
    else {
      self
    }
  }
}


// Return true if the next token in `it` is `kind`
fn peek_token(it: &mut ParseIter, kind: Token) -> bool {
  if let Some(&c) = it.peek() {
    c.node == kind
  }
  else {
    false
  }
}


// Return true if the next token in `it` is `kind` *and* consume the token
fn use_token(it: &mut ParseIter, kind: Token) -> bool {
  if let Some(&c) = it.peek() {
    if c.node == kind {
      it.next();
    }
    c.node == kind
  }
  else {
    false
  }
}


// Panic if the next token in `it` is *not* `kind`
fn require_token(it: &mut ParseIter, kind: Token) {
  if let Some(&c) = it.peek() {
    if c.node != kind {
      panic!("Expected {:?} but found {:?}", kind, c.node);
    }
    it.next();
  } else {
    panic!("Expected {:?} but found end of stream", kind);
  }
}


fn parse_expr(it: &mut ParseIter) -> Parse {
  if let Some(&c) = it.peek() {
    return match c.node {
      Token::Null => {it.next(); Parse::Ok(Node::Null)},
      Token::Bool(x) => {it.next(); Parse::Ok(Node::Bool(x))},
      Token::Float(x) => {it.next(); Parse::Ok(Node::Float(x))},
      Token::Int(x) => {it.next(); Parse::Ok(Node::Int(x))},
      Token::Str(ref x) => {it.next(); Parse::Ok(Node::Str(x.clone()))},
      Token::Name(ref x) => {it.next(); Parse::Ok(Node::Name(x.clone()))},
      _ => Parse::None,
    }
  }
  Parse::None
}


fn parse_stmt(it: &mut ParseIter) -> Parse {
  if let Some(&c) = it.peek() {
    match c.node {
      Token::Break => {
        it.next();
        Parse::Ok(Node::Break)
      }

      Token::Continue => {
        it.next();
        Parse::Ok(Node::Continue)
      }

      Token::Pass => {
        it.next();
        Parse::Ok(Node::Pass)
      }

      Token::If => {
        it.next();
        parse_expr(it).map(|expr| Node::If{cond: Box::new(expr)})
      }

      _ => {
        parse_expr(it).map(|expr| Node::Stmt(Box::new(expr)))
      }
    }
  }
  else {
    Parse::None
  }
}

pub fn parse(tokens: Vec<Spanned<Token>>) -> Parse {
  let mut it: ParseIter = tokens.iter().peekable();
  let mut nodes: Vec<Node> = vec![];
  loop {
    match parse_stmt(&mut it) {
      Parse::Ok(node) => nodes.push(node),
      Parse::Err(err) => {return Parse::Err(err)},
      Parse::None => {break},
    }

    require_token(&mut it, Token::Newline);
    if peek_token(&mut it, Token::EOF) {
      break;
    }
  }
  Parse::Ok(Node::Block(nodes))
}
