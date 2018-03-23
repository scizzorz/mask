use codemap::Spanned;
use lexer::Token;
use std::iter::Peekable;
use std::slice::Iter;
use self::ParseErrorKind::*;

type ParseIter<'a> = Peekable<Iter<'a, Spanned<Token>>>;
type Parse = Result<Node, ParseErrorKind>;

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
  Table,
}


#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
  UnexpectedToken(Token),
  UnexpectedEOF,
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
fn require_token(it: &mut ParseIter, kind: Token) -> Result<(), ParseErrorKind> {
  if let Some(&c) = it.peek() {
    it.next();

    if c.node == kind {
      return Ok(());
    }

    return Err(UnexpectedToken(c.node.clone()));
  }

  return Err(UnexpectedEOF);
}


fn parse_expr(it: &mut ParseIter) -> Parse {
  if let Some(&c) = it.peek() {
    return match c.node {
      Token::Null => {it.next(); Ok(Node::Null)},
      Token::Bool(x) => {it.next(); Ok(Node::Bool(x))},
      Token::Float(x) => {it.next(); Ok(Node::Float(x))},
      Token::Int(x) => {it.next(); Ok(Node::Int(x))},
      Token::Str(ref x) => {it.next(); Ok(Node::Str(x.clone()))},
      Token::Name(ref x) => {it.next(); Ok(Node::Name(x.clone()))},
      _ => Err(UnexpectedToken(c.node.clone())),
    }
  }

  Err(UnexpectedEOF)
}


fn parse_stmt(it: &mut ParseIter) -> Parse {
  if let Some(&c) = it.peek() {
    return match c.node {
      Token::Break => {
        it.next();
        Ok(Node::Break)
      }

      Token::Continue => {
        it.next();
        Ok(Node::Continue)
      }

      Token::Pass => {
        it.next();
        Ok(Node::Pass)
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

  Err(UnexpectedEOF)
}

pub fn parse(tokens: Vec<Spanned<Token>>) -> Parse {
  let mut it: ParseIter = tokens.iter().peekable();
  let mut nodes: Vec<Node> = vec![];

  while !peek_token(&mut it, Token::EOF) {
    nodes.push(parse_stmt(&mut it)?);
    require_token(&mut it, Token::End)?;
  }

  Ok(Node::Block(nodes))
}
