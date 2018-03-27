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

  BinExpr {
    lhs: Box<Node>,
    op: Token,
    rhs: Box<Node>,
  },

  UnExpr {
    val: Box<Node>,
    op: Token,
  },

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
pub enum Op {
  Right(u32),
  Left(u32),
  None,
}


#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
  UnexpectedToken(Token),
  UnexpectedEOF,
  UnknownBinaryOperator,
  UnknownUnaryOperator,
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


fn op_precedence(op: &Token) -> Op {
  match *op {
    Token::Add | Token::Sub => Op::Left(10),
    Token::Div | Token::Mul => Op::Left(20),
    Token::Car => Op::Right(30),
    _ => Op::None,
  }
}


fn parse_binexpr(it: &mut ParseIter) -> Parse {
  let mut expr = parse_unexpr(it)?;

  while let Some(&c) = it.peek() {
    let prec = op_precedence(&c.node);

    if let Op::None = prec {
      break;
    }

    it.next();

    let rhs = parse_unexpr(it)?;

    expr = if let Node::BinExpr{lhs: cur_lhs, op: cur_op, rhs: cur_rhs} = expr.clone() {
      let cur_prec = op_precedence(&cur_op);
      match (cur_prec, prec) {
        // these should never happen
        (_, Op::None) => {break}
        (Op::None, _) => {break}

        // left-to-right
        // there has to be a better way to handle this, no?
        (Op::Left(n), Op::Left(m)) if n >= m => {
          Node::BinExpr {lhs: Box::new(expr), op: c.node.clone(), rhs: Box::new(rhs)}
        }
        (Op::Right(n), Op::Right(m)) if n > m => {
          Node::BinExpr {lhs: Box::new(expr), op: c.node.clone(), rhs: Box::new(rhs)}
        }
        (Op::Right(n), Op::Left(m)) if n >= m => {
          Node::BinExpr {lhs: Box::new(expr), op: c.node.clone(), rhs: Box::new(rhs)}
        }
        (Op::Left(n), Op::Right(m)) if n >= m => {
          Node::BinExpr {lhs: Box::new(expr), op: c.node.clone(), rhs: Box::new(rhs)}
        }

        // right-to-left
        _ => {
          Node::BinExpr {
            lhs: cur_lhs,
            op: cur_op,
            rhs: Box::new(Node::BinExpr {lhs: cur_rhs, op: c.node.clone(), rhs: Box::new(rhs)}),
          }
        }
      }
    }
    else {
      Node::BinExpr {lhs: Box::new(expr), op: c.node.clone(), rhs: Box::new(rhs)}
    };
  };

  Ok(expr)
}


fn parse_unexpr(it: &mut ParseIter) -> Parse {
  parse_simple(it)
}


fn parse_simple(it: &mut ParseIter) -> Parse {
  parse_prefix(it)
}


fn parse_prefix(it: &mut ParseIter) -> Parse {
  if let Some(&c) = it.peek() {
    return match c.node {
      Token::Null => {it.next(); Ok(Node::Null)},
      Token::Bool(x) => {it.next(); Ok(Node::Bool(x))},
      Token::Float(x) => {it.next(); Ok(Node::Float(x))},
      Token::Int(x) => {it.next(); Ok(Node::Int(x))},
      Token::Str(ref x) => {it.next(); Ok(Node::Str(x.clone()))},
      Token::Name(ref x) => {it.next(); Ok(Node::Name(x.clone()))},
      Token::Table => {it.next(); Ok(Node::Table)},
      Token::Pal => {
        it.next();
        let out = parse_binexpr(it)?;
        require_token(it, Token::Par)?;
        Ok(out)
      }
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

      _ => {
        parse_binexpr(it).map(|expr| Node::Stmt(Box::new(expr)))
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
