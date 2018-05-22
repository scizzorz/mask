use self::ParseErrorKind::*;
use codemap::Span;
use codemap::Spanned;
use float;
use int;
use lexer::Token;
use std::iter::Peekable;
use std::slice::Iter;

type ParseIter<'a> = Peekable<Iter<'a, Spanned<Token>>>;
type Parse = Result<Node, ParseErrorKind>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Var {
  Single(String),
  Multi(Vec<Var>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Place {
  Single(Box<Node>),
  Multi(Vec<Place>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Node {
  // Generic
  Expr(Box<Node>),
  Block(Vec<Node>),

  // Simple statements
  Return(Option<Box<Node>>),
  Break,
  Continue,
  Pass,

  // Compound statements
  Assn {
    lhs: Place,
    rhs: Box<Node>,
  },

  Catch {
    body: Box<Node>,
  },

  If {
    cond: Box<Node>,
    body: Box<Node>,
    els: Option<Box<Node>>,
  },

  ElseIf {
    cond: Box<Node>,
    body: Box<Node>,
  },

  Else {
    body: Box<Node>,
  },

  For {
    decl: Var,
    expr: Box<Node>,
    body: Box<Node>,
  },

  While {
    expr: Box<Node>,
    body: Box<Node>,
  },

  Loop {
    body: Box<Node>,
  },

  // Copound expressions
  FuncDef {
    params: Vec<String>,
    body: Box<Node>,
  },

  Index {
    lhs: Box<Node>,
    rhs: Box<Node>,
  },

  MethodCall {
    owner: Box<Node>,
    method: Box<Node>,
    args: Vec<Node>,
  },

  FuncCall {
    func: Box<Node>,
    args: Vec<Node>,
  },

  BinExpr {
    lhs: Box<Node>,
    op: Token,
    rhs: Box<Node>,
  },

  CmpExpr {
    nodes: Vec<Node>,
    ops: Vec<Token>,
  },

  LogicExpr {
    nodes: Vec<Node>,
    ops: Vec<Token>,
  },

  UnExpr {
    val: Box<Node>,
    op: Token,
  },
  Super(usize, Box<Node>),

  // Literals
  Null,
  Bool(bool),
  #[serde(with = "::FloatDef")]
  Float(float),
  Int(int),
  Str(String),
  Name(String),
  Local,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Op {
  Right(u32),
  Left(u32),
  None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Token::Cat => Op::Left(10),
    Token::Add | Token::Sub => Op::Left(20),
    Token::Div | Token::Mul => Op::Left(30),
    Token::Car => Op::Right(40),
    Token::Sup => Op::Right(50),
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
        Ok(Node::FuncDef {
          params: params,
          body: Box::new(body),
        })
      }
      Token::Catch => {
        it.next();
        let block = parse_block(it)?;
        Ok(Node::Catch {
          body: Box::new(block),
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
      Token::Pipe => {
        it.next();
        let params = parse_fn_params(it)?;
        require_token(it, Token::Pipe)?;
        let expr = parse_il_expr(it)?;
        Ok(Node::FuncDef {
          params: params,
          body: Box::new(Node::Return(Some(Box::new(expr)))),
        })
      }
      _ => parse_logic_expr(it),
    };
  }

  Err(UnexpectedEOF)
}

fn parse_logic_expr(it: &mut ParseIter) -> Parse {
  let mut expr = parse_cmp_expr(it)?;

  let mut break_left = false;

  while let Some(&tok) = it.peek() {
    match tok.node {
      Token::And | Token::Or => {
        it.next();
        let new = parse_cmp_expr(it)?;
        if !break_left {
          expr = Node::LogicExpr {
            nodes: vec![expr, new],
            ops: vec![tok.node.clone()],
          };

          break_left = true;
        } else if let Node::LogicExpr {
          ref mut nodes,
          ref mut ops,
        } = expr
        {
          nodes.push(new);
          ops.push(tok.node.clone());
        }
      }
      _ => break,
    }
  }

  Ok(expr)
}

fn parse_cmp_expr(it: &mut ParseIter) -> Parse {
  let mut expr = parse_bin_expr(it)?;
  let mut break_left = false;

  while let Some(&tok) = it.peek() {
    match tok.node {
      Token::Eql | Token::Ne | Token::Ge | Token::Gt | Token::Le | Token::Lt => {
        it.next();
        let new = parse_bin_expr(it)?;
        if !break_left {
          expr = Node::CmpExpr {
            nodes: vec![expr, new],
            ops: vec![tok.node.clone()],
          };

          break_left = true;
        } else if let Node::CmpExpr {
          ref mut nodes,
          ref mut ops,
        } = expr
        {
          nodes.push(new);
          ops.push(tok.node.clone());
        }
      }

      _ => break,
    }
  }

  Ok(expr)
}

fn parse_bin_expr(it: &mut ParseIter) -> Parse {
  let mut expr = parse_un_expr(it)?;

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

    expr = match (break_left, expr.clone()) {
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

  Ok(expr)
}

fn parse_un_expr(it: &mut ParseIter) -> Parse {
  if let Some(&tok) = it.peek() {
    return match tok.node {
      Token::Sub | Token::Not | Token::Neg | Token::Mul | Token::Cat => {
        it.next();
        let val = parse_un_expr(it)?;
        Ok(Node::UnExpr {
          op: tok.node.clone(),
          val: Box::new(val),
        })
      }
      _ => parse_super(it),
    };
  }

  Err(UnexpectedEOF)
}

fn parse_super(it: &mut ParseIter) -> Parse {
  if let Some(&tok) = it.peek() {
    return match tok.node {
      Token::Dot => {
        let mut count: usize = 0;
        while use_token(it, Token::Dot) {
          count += 1;
        }
        let idx = parse_name_as_str(it)?;
        Ok(Node::Super(count, Box::new(idx)))
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

fn parse_fn_params(it: &mut ParseIter) -> Result<Vec<String>, ParseErrorKind> {
  let mut params: Vec<String> = Vec::new();
  while let Some(&tok) = it.peek() {
    match tok.node {
      Token::Name(ref x) => {
        it.next();
        params.push(x.to_string());
        if !use_token(it, Token::Com) {
          break;
        }
      }
      _ => break,
    }
  }

  Ok(params)
}

fn parse_fn_args(it: &mut ParseIter) -> Result<Vec<Node>, ParseErrorKind> {
  let mut args = Vec::new();
  require_token(it, Token::Pal)?;
  while !peek_token(it, Token::Par) {
    let arg = parse_il_expr(it)?;
    args.push(arg);
    if !use_token(it, Token::Com) {
      break;
    }
  }
  require_token(it, Token::Par)?;
  Ok(args)
}

fn parse_simple(it: &mut ParseIter) -> Parse {
  let mut atom = parse_atom(it)?;
  while let Some(&tok) = it.peek() {
    match tok.node {
      Token::Col => {
        it.next();
        let method = parse_name_as_str(it)?;
        let args = parse_fn_args(it)?;
        atom = Node::MethodCall {
          owner: Box::new(atom),
          method: Box::new(method),
          args: args,
        };
      }

      Token::Pal => {
        let args = parse_fn_args(it)?;
        atom = Node::FuncCall {
          func: Box::new(atom),
          args: args,
        };
      }

      Token::Sql => {
        it.next();
        let idx = parse_logic_expr(it)?;
        require_token(it, Token::Sqr)?;
        atom = Node::Index {
          lhs: Box::new(atom),
          rhs: Box::new(idx),
        };
      }

      Token::Dot => {
        it.next();
        let idx = parse_name_as_str(it)?;
        atom = Node::Index {
          lhs: Box::new(atom),
          rhs: Box::new(idx),
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
        let out = parse_logic_expr(it)?;
        require_token(it, Token::Par)?;
        Ok(out)
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
        Ok(Node::Str(x.clone()))
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
        Ok(Node::Name(x.clone()))
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
        Ok(Node::Null)
      }
      Token::Bool(x) => {
        it.next();
        Ok(Node::Bool(x))
      }
      Token::Float(x) => {
        it.next();
        Ok(Node::Float(x))
      }
      Token::Int(x) => {
        it.next();
        Ok(Node::Int(x))
      }
      Token::Str(ref x) => {
        it.next();
        Ok(Node::Str(x.clone()))
      }
      Token::Name(ref x) => {
        it.next();
        Ok(Node::Name(x.clone()))
      }
      Token::Local => {
        it.next();
        Ok(Node::Local)
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
        Ok(Node::Assn {
          lhs: place,
          rhs: Box::new(rhs),
        })
      }

      _ => match place {
        Place::Single(bx) => Ok(Node::Expr(bx)),
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
        Ok(Node::Break)
      }

      Token::Continue => {
        it.next();
        Ok(Node::Continue)
      }

      Token::If => {
        it.next();
        let cond = parse_logic_expr(it)?;
        let body = parse_block(it)?;
        Ok(Node::If {
          cond: Box::new(cond),
          body: Box::new(body),
          els: None,
        })
      }

      Token::Else => {
        it.next();
        if use_token(it, Token::If) {
          let cond = parse_logic_expr(it)?;
          let body = parse_block(it)?;
          Ok(Node::ElseIf {
            cond: Box::new(cond),
            body: Box::new(body),
          })
        } else {
          let body = parse_block(it)?;
          Ok(Node::Else {
            body: Box::new(body),
          })
        }
      }

      Token::For => {
        it.next();
        let decl = parse_decl(it)?;
        require_token(it, Token::In)?;
        let expr = parse_il_expr(it)?;
        let body = parse_block(it)?;
        Ok(Node::For {
          decl: decl,
          expr: Box::new(expr),
          body: Box::new(body),
        })
      }

      Token::While => {
        it.next();
        let expr = parse_il_expr(it)?;
        let body = parse_block(it)?;
        Ok(Node::While {
          expr: Box::new(expr),
          body: Box::new(body),
        })
      }

      Token::Loop => {
        it.next();
        let body = parse_block(it)?;
        Ok(Node::Loop {
          body: Box::new(body),
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
        Ok(Node::Return(val))
      }

      Token::Pass => {
        it.next();
        Ok(Node::Pass)
      }

      Token::Func | Token::Catch => parse_ml_expr(it).map(|expr| Node::Expr(Box::new(expr))),

      _ => parse_assn(it),
    };
  }

  Err(UnexpectedEOF)
}

fn parse_block(it: &mut ParseIter) -> Parse {
  let mut nodes: Vec<Node> = vec![];

  require_token(it, Token::Enter)?;

  while !peek_token(it, Token::Exit) {
    let stmt = parse_stmt(it)?;
    nodes.push(stmt);
    require_token(it, Token::End)?;
  }

  require_token(it, Token::Exit)?;

  Ok(Node::Block(nodes))
}

pub fn parse(tokens: Vec<Spanned<Token>>) -> Parse {
  let mut it: ParseIter = tokens.iter().peekable();
  let mut nodes: Vec<Node> = vec![];

  while !peek_token(&mut it, Token::EOF) {
    let stmt = parse_stmt(&mut it)?;
    nodes.push(stmt);
    require_token(&mut it, Token::End)?;
  }

  Ok(Node::Block(nodes))
}

#[cfg(test)]
#[path = "./tests/parser.rs"]
mod tests;
