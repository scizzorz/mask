use codemap::Spanned;
use codemap::Span;
use lexer::Token;
use std::iter::Peekable;
use std::slice::Iter;
use self::ParseErrorKind::*;

type ParseIter<'a> = Peekable<Iter<'a, Spanned<Token>>>;
type Parse = Result<Spanned<Node>, ParseErrorKind>;

#[derive(Debug, Clone, PartialEq)]
pub enum Var {
  Single(String),
  Multi(Vec<Var>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Place {
  Single(Box<Node>),
  Multi(Vec<Place>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
  Block(Vec<Spanned<Node>>),
  Stmt(Box<Spanned<Node>>),
  Catch(Vec<Spanned<Node>>),
  Assn {
    lhs: Place,
    rhs: Box<Spanned<Node>>,
  },
  If {
    cond: Box<Spanned<Node>>,
    body: Vec<Spanned<Node>>,
    els: Option<Box<Spanned<Node>>>,
  },
  ElseIf {
    cond: Box<Spanned<Node>>,
    body: Vec<Spanned<Node>>,
  },
  Else {
    body: Vec<Spanned<Node>>,
  },
  For {
    decl: Var,
    expr: Box<Spanned<Node>>,
    body: Vec<Spanned<Node>>,
  },
  While {
    expr: Box<Spanned<Node>>,
    body: Vec<Spanned<Node>>,
  },
  Loop {
    body: Vec<Spanned<Node>>,
  },
  Return(Option<Box<Spanned<Node>>>),
  Break,
  Continue,
  Expr,
  Pass,
  Index {
    lhs: Box<Spanned<Node>>,
    rhs: Box<Spanned<Node>>,
  },

  Method {
    owner: Box<Spanned<Node>>,
    method: Box<Spanned<Node>>,
    args: Spanned<Vec<Spanned<Node>>>,
  },

  Func {
    params: Vec<String>,
    body: Vec<Spanned<Node>>,
  },

  Lambda {
    params: Vec<String>,
    expr: Box<Spanned<Node>>,
  },

  Call {
    func: Box<Spanned<Node>>,
    args: Spanned<Vec<Spanned<Node>>>,
  },

  BinExpr {
    lhs: Box<Spanned<Node>>,
    op: Token,
    rhs: Box<Spanned<Node>>,
  },

  UnExpr {
    val: Box<Spanned<Node>>,
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
  UnusedPlaces,
}

// Return true if the next token in `it` is `kind`
fn peek_token(it: &mut ParseIter, kind: Token) -> bool {
  if let Some(&tok) = it.peek() {
    tok.node == kind
  } else {
    false
  }
}

// Return true if the next token in `it` is `kind` *and* consume the token
fn use_token(it: &mut ParseIter, kind: Token) -> bool {
  if let Some(&tok) = it.peek() {
    if tok.node == kind {
      it.next();
    }
    tok.node == kind
  } else {
    false
  }
}

// Panic if the next token in `it` is *not* `kind`
fn require_token(it: &mut ParseIter, kind: Token) -> Result<Span, ParseErrorKind> {
  if let Some(&tok) = it.peek() {
    if tok.node == kind {
      it.next();
      return Ok(tok.span);
    }

    return Err(UnexpectedToken(tok.node.clone()));
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

fn parse_ml_expr(it: &mut ParseIter) -> Parse {
  if let Some(&tok) = it.peek() {
    return match tok.node {
      Token::Func => {
        it.next();
        require_token(it, Token::Pal)?;
        let params = parse_fn_params(it)?;
        require_token(it, Token::Par)?;
        let body = parse_block(it)?;
        Ok(Spanned {
          node: Node::Func {
            params: params,
            body: body,
          },
          span: tok.span.merge(body.span),
        })
      }
      Token::Catch => {
        it.next();
        let block = parse_block(it)?;
        Ok(Spanned {
          node: Node::Catch(block),
          span: tok.span.merge(block.span),
        })
      }
      _ => parse_il_expr(it),
    };
  }

  Err(UnexpectedEOF)
}

fn parse_il_expr(it: &mut ParseIter) -> Parse {
  if let Some(&tok) = it.peek() {
    return match tok.node {
      Token::Or => {
        it.next();
        let params = parse_fn_params(it)?;
        require_token(it, Token::Or)?;
        let expr = parse_il_expr(it)?;
        Ok(Spanned {
          node: Node::Lambda {
            params: params,
            expr: Box::new(expr),
          },
          span: tok.span.merge(expr.span),
        })
      }
      _ => parse_bin_expr(it),
    };
  }

  Err(UnexpectedEOF)
}

fn parse_bin_expr(it: &mut ParseIter) -> Parse {
  let mut expr = parse_un_expr(it)?;
  let start_span = expr.span;

  // prevents this from breaking the LHS until we know we made it
  // otherwise, things like (2 + 3) * 4 get restructured into 2 + (3 * 4)
  let mut break_left = false;

  while let Some(&tok) = it.peek() {
    let prec = op_precedence(&tok.node);

    if let Op::None = prec {
      break;
    }

    it.next();

    let rhs = parse_un_expr(it)?;

    expr.node = match (break_left, expr.node.clone()) {
      (
        true,
        Node::BinExpr {
          lhs: cur_lhs,
          op: cur_op,
          rhs: cur_rhs,
        },
      ) => {
        let cur_prec = op_precedence(&cur_op);
        match (cur_prec, prec) {
          // these should never happen
          (_, Op::None) => break,
          (Op::None, _) => break,

          // left-to-right
          // there has to be a better way to handle this, no?
          (Op::Left(n), Op::Left(m)) if n >= m => Node::BinExpr {
            lhs: Box::new(expr),
            op: tok.node.clone(),
            rhs: Box::new(rhs),
          },
          (Op::Right(n), Op::Right(m)) if n > m => Node::BinExpr {
            lhs: Box::new(expr),
            op: tok.node.clone(),
            rhs: Box::new(rhs),
          },
          (Op::Right(n), Op::Left(m)) if n >= m => Node::BinExpr {
            lhs: Box::new(expr),
            op: tok.node.clone(),
            rhs: Box::new(rhs),
          },
          (Op::Left(n), Op::Right(m)) if n >= m => Node::BinExpr {
            lhs: Box::new(expr),
            op: tok.node.clone(),
            rhs: Box::new(rhs),
          },

          // right-to-left
          _ => Node::BinExpr {
            lhs: cur_lhs,
            op: cur_op,
            rhs: Box::new(Node::BinExpr {
              lhs: cur_rhs,
              op: tok.node.clone(),
              rhs: Box::new(rhs),
            }),
          },
        }
      }
      _ => Node::BinExpr {
        lhs: Box::new(expr),
        op: tok.node.clone(),
        rhs: Box::new(rhs),
      },
    };

    break_left = true;
  }

  Ok(Spanned {
    node: expr,
    span: start_span, // FIXME this isn't the right span
  })
}

fn parse_un_expr(it: &mut ParseIter) -> Parse {
  if let Some(&tok) = it.peek() {
    return match tok.node {
      Token::Sub | Token::Not | Token::Neg => {
        it.next();
        let val = parse_un_expr(it)?;
        Ok(Spanned {
          node: Node::UnExpr {
            op: tok.node.clone(),
            val: Box::new(val),
          },
          span: tok.span.merge(val.span),
        })
      }
      _ => parse_simple(it),
    };
  }

  Err(UnexpectedEOF)
}

/* unused, here for reference
fn parse_index(it: &mut ParseIter) -> Parse {
  if let Some(&c) = it.peek() {
    return match c.node {
      Token::Sql => {
        it.next();
        let idx = parse_bin_expr(it)?;
        require_token(it, Token::Sqr)?;
        Ok(idx)
      }
      Token::Dot => {
        it.next();
        parse_name_as_str(it)
      }
      _ => {
        Err(UnexpectedToken(c.node.clone()))
      }
    };
  }

  Err(UnexpectedEOF)
}
*/

fn parse_fn_params(it: &mut ParseIter) -> Result<Vec<Spanned<String>>, ParseErrorKind> {
  let mut params = Vec::new();
  while let Some(&tok) = it.peek() {
    match tok.node {
      Token::Name(ref x) => {
        it.next();
        params.push(Spanned {
          node: x.to_string(),
          span: tok.span,
        });

        if !use_token(it, Token::Com) {
          break;
        }
      }
      _ => break,
    }
  }

  Ok(params);
}

fn parse_fn_args(it: &mut ParseIter) -> Result<Spanned<Vec<Spanned<Node>>>, ParseErrorKind> {
  let mut args = Vec::new();
  let start_span = require_token(it, Token::Pal)?;
  while !peek_token(it, Token::Par) {
    let arg = parse_il_expr(it)?;
    args.push(arg);
    if !use_token(it, Token::Com) {
      break;
    }
  }
  let end_span = require_token(it, Token::Par)?;
  Ok(Spanned {
    node: args,
    span: start_span.merge(end_span),
  })
}

fn parse_simple(it: &mut ParseIter) -> Parse {
  let mut atom = parse_atom(it)?;
  while let Some(&tok) = it.peek() {
    match tok.node {
      Token::Col => {
        it.next();
        let method = parse_name_as_str(it)?;
        let args = parse_fn_args(it)?;
        atom = Spanned {
          node: Node::Method {
            owner: Box::new(atom),
            method: Box::new(method),
            args: args,
          },
          span: atom.span.merge(args.span),
        };
      }

      Token::Pal => {
        let args = parse_fn_args(it)?;
        atom = Spanned {
          node: Node::Call {
            func: Box::new(atom),
            args: args,
          },
          span: atom.span.merge(args.span),
        };
      }

      Token::Sql => {
        it.next();
        let idx = parse_bin_expr(it)?;
        let end_span = require_token(it, Token::Sqr)?;
        atom = Spanned {
          node: Node::Index {
            lhs: Box::new(atom),
            rhs: Box::new(idx),
          },
          span: atom.span.merge(end_span.span),
        };
      }

      Token::Dot => {
        it.next();
        let idx = parse_name_as_str(it)?;
        atom = Spanned {
          node: Node::Index {
            lhs: Box::new(atom),
            rhs: Box::new(idx),
          },
          span: atom.span.merge(idx.span),
        };
      }

      _ => break,
    }
  }

  Ok(atom)
}

fn parse_atom(it: &mut ParseIter) -> Parse {
  if let Some(&tok) = it.peek() {
    return match tok.node {
      Token::Pal => {
        it.next();
        let out = parse_atom(it)?; // FIXME parse_bin_expr
        let end_span = require_token(it, Token::Par)?;
        Ok(Spanned {
          node: out.node,
          span: tok.span.merge(end_span),
        })
      }
      _ => parse_quark(it),
    };
  }

  Err(UnexpectedEOF)
}

fn parse_name_as_str(it: &mut ParseIter) -> Parse {
  if let Some(&tok) = it.peek() {
    return match tok.node {
      Token::Name(ref x) => {
        it.next();
        Ok(Spanned {
          node: Node::Str(x.clone()),
          span: tok.span,
        })
      }
      ref x => Err(UnexpectedToken(x.clone())),
    };
  }

  Err(UnexpectedEOF)
}

fn parse_name(it: &mut ParseIter) -> Parse {
  if let Some(&tok) = it.peek() {
    return match tok.node {
      Token::Name(ref x) => {
        it.next();
        Ok(Spanned {
          node: Node::Name(x.clone()),
          span: tok.span,
        })
      }
      ref x => Err(UnexpectedToken(x.clone())),
    };
  }

  Err(UnexpectedEOF)
}

fn parse_quark(it: &mut ParseIter) -> Parse {
  if let Some(&tok) = it.peek() {
    return match tok.node {
      Token::Null => {
        it.next();
        Ok(Spanned {
          node: Node::Null,
          span: tok.span,
        })
      }
      Token::Bool(x) => {
        it.next();
        Ok(Spanned {
          node: Node::Bool(x),
          span: tok.span,
        })
      }
      Token::Float(x) => {
        it.next();
        Ok(Spanned {
          node: Node::Float(x),
          span: tok.span,
        })
      }
      Token::Int(x) => {
        it.next();
        Ok(Spanned {
          node: Node::Int(x),
          span: tok.span,
        })
      }
      Token::Str(ref x) => {
        it.next();
        Ok(Spanned {
          node: Node::Str(x.clone()),
          span: tok.span,
        })
      }
      Token::Name(ref x) => {
        it.next();
        Ok(Spanned {
          node: Node::Name(x.clone()),
          span: tok.span,
        })
      }
      Token::Table => {
        it.next();
        Ok(Spanned {
          node: Node::Table,
          span: tok.span,
        })
      }
      ref x => Err(UnexpectedToken(x.clone())),
    };
  }

  Err(UnexpectedEOF)
}

fn parse_decl(it: &mut ParseIter) -> Result<Var, ParseErrorKind> {
  if let Some(&tok) = it.peek() {
    return match tok.node {
      Token::Sql => {
        it.next();
        let mut pieces: Vec<Var> = Vec::new();
        loop {
          let new_piece = parse_decl(it)?;
          pieces.push(new_piece);
          if !use_token(it, Token::Com) {
            break;
          }
        }
        require_token(it, Token::Sqr)?;
        Ok(Var::Multi(pieces))
      }
      Token::Name(ref x) => {
        it.next();
        Ok(Var::Single(x.clone()))
      }
      ref x => Err(UnexpectedToken(x.clone())),
    };
  }

  Err(UnexpectedEOF)
}

fn parse_place(it: &mut ParseIter) -> Result<Place, ParseErrorKind> {
  if let Some(&tok) = it.peek() {
    return match tok.node {
      Token::Sql => {
        it.next();
        let mut pieces: Vec<Place> = Vec::new();
        loop {
          let new_piece = parse_place(it)?;
          pieces.push(new_piece);
          if !use_token(it, Token::Com) {
            break;
          }
        }
        require_token(it, Token::Sqr)?;
        Ok(Place::Multi(pieces))
      }

      _ => {
        let node = parse_il_expr(it)?;
        Ok(Place::Single(Box::new(node)))
      }
    };
  }

  Err(UnexpectedEOF)
}

fn parse_assn(it: &mut ParseIter) -> Parse {
  let place = parse_place(it)?;

  if let Some(&tok) = it.peek() {
    return match tok.node {
      Token::Ass => {
        it.next();
        let rhs = parse_ml_expr(it)?;
        Ok(Spanned {
          node: Node::Assn {
            lhs: place,
            rhs: Box::new(rhs),
          },
          span: tok.span.merge(rhs.span), // FIXME this isn't the right span
        })
      }

      _ => match place {
        Place::Single(bx) => Ok(Spanned {
          node: Node::Stmt(bx),
          span: tok.span, // FIXME this isn't the right span
        }),
        Place::Multi(_) => Err(UnusedPlaces),
      },
    };
  }

  Err(UnexpectedEOF)
}

fn parse_stmt(it: &mut ParseIter) -> Parse {
  if let Some(&tok) = it.peek() {
    return match tok.node {
      Token::Break => {
        it.next();
        Ok(Spanned {
          node: Node::Break,
          span: tok.span,
        })
      }

      Token::Continue => {
        it.next();
        Ok(Spanned {
          node: Node::Continue,
          span: tok.span,
        })
      }

      Token::If => {
        it.next();
        let cond = parse_bin_expr(it)?;
        let body = parse_block(it)?;
        Ok(Spanned {
          node: Node::If {
            cond: Box::new(cond),
            body: body,
            els: None,
          },
          span: tok.span.merge(body.span),
        })
      }

      Token::Else => {
        it.next();
        if use_token(it, Token::If) {
          let cond = parse_bin_expr(it)?;
          let body = parse_block(it)?;
          Ok(Spanned {
            node: Node::ElseIf {
              cond: Box::new(cond),
              body: body,
            },
            span: tok.span.merge(body.span),
          })
        } else {
          let body = parse_block(it)?;
          Ok(Spanned {
            node: Node::Else { body: body },
            span: tok.span.merge(body),
          })
        }
      }

      Token::For => {
        it.next();
        let decl = parse_decl(it)?;
        require_token(it, Token::In)?;
        let expr = parse_il_expr(it)?;
        let body = parse_block(it)?;
        Ok(Spanned {
          node: Node::For {
            decl: decl,
            expr: Box::new(expr),
            body: body,
          },
          span: tok.span.merge(body.span),
        })
      }

      Token::While => {
        it.next();
        let expr = parse_il_expr(it)?;
        let body = parse_block(it)?;
        Ok(Spanned {
          node: Node::While {
            expr: Box::new(expr),
            body: body,
          },
          span: tok.span.merge(body.span),
        })
      }

      Token::Loop => {
        it.next();
        let body = parse_block(it)?;
        Ok(Spanned {
          node: Node::Loop { body: body },
          span: tok.span.merge(body.span),
        })
      }

      Token::Return => {
        it.next();
        let val = if peek_token(it, Token::End) {
          None
        } else {
          let val = parse_ml_expr(it)?;
          Some(Box::new(val))
        };
        Ok(Spanned {
          node: Node::Return(val),
          span: tok.span,
        })
      }

      Token::Pass => {
        it.next();
        Ok(Spanned {
          node: Node::Pass,
          span: tok.span,
        })
      }

      Token::Func | Token::Catch => parse_ml_expr(it).map(|expr| Spanned {
        node: Node::Stmt(Box::new(expr.node)),
        span: expr.span,
      }),

      _ => parse_assn(it),
    };
  }

  Err(UnexpectedEOF)
}

fn parse_block(it: &mut ParseIter) -> Result<Spanned<Vec<Spanned<Node>>>, ParseErrorKind> {
  let mut nodes = Vec::new();

  let start_span = require_token(it, Token::Enter)?;

  while !peek_token(it, Token::Exit) {
    let stmt = parse_stmt(it)?;
    nodes.push(stmt);
    require_token(it, Token::End)?;
  }

  let end_span = require_token(it, Token::Exit)?;

  Ok(Spanned {
    node: nodes,
    span: start_span.merge(end_span),
  })
}

pub fn parse(tokens: Vec<Spanned<Token>>) -> Parse {
  let mut it: ParseIter = tokens.iter().peekable();
  let mut nodes: Vec<Node> = vec![];

  while !peek_token(&mut it, Token::EOF) {
    /*
    let stmt = parse_stmt(&mut it)?;
    nodes.push(stmt);
    */
    require_token(&mut it, Token::End)?;
  }

  let span = require_token(&mut it, Token::EOF)?;

  Ok(Spanned {
    node: Node::Block(nodes),
    span: span,
  })
}

#[cfg(test)]
#[path = "./tests/parser.rs"]
mod tests;
