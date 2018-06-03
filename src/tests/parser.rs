use super::super::lexer;
use super::*;
use codemap::CodeMap;
use codemap::Spanned;
use std::cmp::PartialEq;
use std::fmt::Debug;

fn get_tokens(source: &str) -> Vec<Spanned<Token>> {
  let mut map = CodeMap::new();
  let file = map.add_file(String::from("_test"), String::from(source));
  lexer::lex(&file).unwrap()
}

fn test_parse<T: Debug + PartialEq>(
  source: &str,
  func: &Fn(&mut ParseIter) -> Result<T, ParseErrorKind>,
  expect: Result<T, ParseErrorKind>,
) {
  let tokens = get_tokens(source);
  let mut it = tokens.iter().peekable();

  assert_eq!(func(&mut it), expect);

  assert_eq!(func(&mut it), Err(UnexpectedToken(lexer::Token::End)));
  it.next();

  assert_eq!(func(&mut it), Err(UnexpectedToken(lexer::Token::EOF)));
  it.next();

  assert_eq!(func(&mut it), Err(UnexpectedEOF));
}

#[test]
fn test_quark() {
  test_parse("null", &parse_quark, Ok(Node::Null));
  test_parse("true", &parse_quark, Ok(Node::Bool(true)));
  test_parse("false", &parse_quark, Ok(Node::Bool(false)));
  test_parse("1.3", &parse_quark, Ok(Node::Float(float::from(1.3))));
  test_parse("0.3", &parse_quark, Ok(Node::Float(float::from(0.3))));
  test_parse("2", &parse_quark, Ok(Node::Int(2)));
  test_parse("3", &parse_quark, Ok(Node::Int(3)));
  test_parse("name", &parse_quark, Ok(Node::Name(String::from("name"))));
}

#[test]
fn test_atom() {
  test_parse("null", &parse_atom, Ok(Node::Null));

  test_parse("(null)", &parse_atom, Ok(Node::Null));
}

#[test]
fn test_simple() {
  test_parse("foo", &parse_simple, Ok(Node::Name(String::from("foo"))));
  test_parse(
    "foo.bar",
    &parse_simple,
    Ok(Node::Index {
      lhs: Box::new(Node::Name(String::from("foo"))),
      rhs: Box::new(Node::Str(String::from("bar"))),
    }),
  );
  test_parse(
    "foo[bar]",
    &parse_simple,
    Ok(Node::Index {
      lhs: Box::new(Node::Name(String::from("foo"))),
      rhs: Box::new(Node::Name(String::from("bar"))),
    }),
  );
  test_parse(
    "foo()",
    &parse_simple,
    Ok(Node::FuncCall {
      func: Box::new(Node::Name(String::from("foo"))),
      args: Vec::new(),
    }),
  );
  test_parse(
    "foo:bar()",
    &parse_simple,
    Ok(Node::MethodCall {
      owner: Box::new(Node::Name(String::from("foo"))),
      method: Box::new(Node::Str(String::from("bar"))),
      args: Vec::new(),
    }),
  );
  test_parse(
    "foo.bar()",
    &parse_simple,
    Ok(Node::FuncCall {
      func: Box::new(Node::Index {
        lhs: Box::new(Node::Name(String::from("foo"))),
        rhs: Box::new(Node::Str(String::from("bar"))),
      }),
      args: Vec::new(),
    }),
  );
  test_parse(
    "foo.bar[baz]:qux()",
    &parse_simple,
    Ok(Node::MethodCall {
      owner: Box::new(Node::Index {
        lhs: Box::new(Node::Index {
          lhs: Box::new(Node::Name(String::from("foo"))),
          rhs: Box::new(Node::Str(String::from("bar"))),
        }),
        rhs: Box::new(Node::Name(String::from("baz"))),
      }),
      method: Box::new(Node::Str(String::from("qux"))),
      args: Vec::new(),
    }),
  );
}

#[test]
fn test_super() {
  test_parse("foo", &parse_super, Ok(Node::Name(String::from("foo"))));
  test_parse(
    ".foo",
    &parse_super,
    Ok(Node::Super(1, Box::new(Node::Str(String::from("foo"))))),
  );
  test_parse(
    "..foo",
    &parse_super,
    Ok(Node::Super(2, Box::new(Node::Str(String::from("foo"))))),
  );
}

#[test]
fn test_fn_args() {
  test_parse("()", &parse_fn_args, Ok(Vec::new()));
  test_parse(
    "(x)",
    &parse_fn_args,
    Ok(vec![Node::Name(String::from("x"))]),
  );
  test_parse(
    "(x,)",
    &parse_fn_args,
    Ok(vec![Node::Name(String::from("x"))]),
  );
  test_parse(
    "(x,y)",
    &parse_fn_args,
    Ok(vec![
      Node::Name(String::from("x")),
      Node::Name(String::from("y")),
    ]),
  );
  test_parse(
    "(x,y,)",
    &parse_fn_args,
    Ok(vec![
      Node::Name(String::from("x")),
      Node::Name(String::from("y")),
    ]),
  );
}

#[test]
fn test_un_expr() {
  test_parse("5", &parse_un_expr, Ok(Node::Int(5)));

  test_parse(
    "foo()",
    &parse_un_expr,
    Ok(Node::FuncCall {
      func: Box::new(Node::Name(String::from("foo"))),
      args: Vec::new(),
    }),
  );

  test_parse(
    "-5",
    &parse_un_expr,
    Ok(Node::UnExpr {
      op: lexer::Token::Sub,
      val: Box::new(Node::Int(5)),
    }),
  );

  test_parse(
    "-foo.bar",
    &parse_un_expr,
    Ok(Node::UnExpr {
      op: lexer::Token::Sub,
      val: Box::new(Node::Index {
        lhs: Box::new(Node::Name(String::from("foo"))),
        rhs: Box::new(Node::Str(String::from("bar"))),
      }),
    }),
  );

  test_parse(
    "!-5",
    &parse_un_expr,
    Ok(Node::UnExpr {
      op: lexer::Token::Not,
      val: Box::new(Node::UnExpr {
        op: lexer::Token::Sub,
        val: Box::new(Node::Int(5)),
      }),
    }),
  );
}

#[test]
fn test_logic_expr() {
  test_parse(
    "1 and 2",
    &parse_logic_expr,
    Ok(Node::LogicExpr {
      nodes: vec![Node::Int(1), Node::Int(2)],
      ops: vec![lexer::Token::And],
    }),
  );

  test_parse(
    "1 or 2",
    &parse_logic_expr,
    Ok(Node::LogicExpr {
      nodes: vec![Node::Int(1), Node::Int(2)],
      ops: vec![lexer::Token::Or],
    }),
  );

  test_parse(
    "1 and 2 or 3",
    &parse_logic_expr,
    Ok(Node::LogicExpr {
      nodes: vec![Node::Int(1), Node::Int(2), Node::Int(3)],
      ops: vec![lexer::Token::And, lexer::Token::Or],
    }),
  );
}

#[test]
fn test_cmp_expr() {
  test_parse(
    "1 < 2",
    &parse_cmp_expr,
    Ok(Node::CmpExpr {
      nodes: vec![Node::Int(1), Node::Int(2)],
      ops: vec![lexer::Token::Lt],
    }),
  );

  test_parse(
    "1 > 2",
    &parse_cmp_expr,
    Ok(Node::CmpExpr {
      nodes: vec![Node::Int(1), Node::Int(2)],
      ops: vec![lexer::Token::Gt],
    }),
  );

  test_parse(
    "1 + 2 > 2 * 4",
    &parse_cmp_expr,
    Ok(Node::CmpExpr {
      nodes: vec![
        Node::BinExpr {
          lhs: Box::new(Node::Int(1)),
          op: Token::Add,
          rhs: Box::new(Node::Int(2)),
        },
        Node::BinExpr {
          lhs: Box::new(Node::Int(2)),
          op: Token::Mul,
          rhs: Box::new(Node::Int(4)),
        },
      ],
      ops: vec![lexer::Token::Gt],
    }),
  );

  test_parse(
    "1 < 2 < 3",
    &parse_cmp_expr,
    Ok(Node::CmpExpr {
      nodes: vec![Node::Int(1), Node::Int(2), Node::Int(3)],
      ops: vec![lexer::Token::Lt, lexer::Token::Lt],
    }),
  );
}

#[test]
fn test_bin_expr() {
  test_parse(
    "1 + 2",
    &parse_bin_expr,
    Ok(Node::BinExpr {
      lhs: Box::new(Node::Int(1)),
      op: lexer::Token::Add,
      rhs: Box::new(Node::Int(2)),
    }),
  );

  test_parse(
    "(1 + 2)",
    &parse_bin_expr,
    Ok(Node::BinExpr {
      lhs: Box::new(Node::Int(1)),
      op: lexer::Token::Add,
      rhs: Box::new(Node::Int(2)),
    }),
  );

  test_parse(
    "1 + 2 * 3",
    &parse_bin_expr,
    Ok(Node::BinExpr {
      lhs: Box::new(Node::Int(1)),
      op: lexer::Token::Add,
      rhs: Box::new(Node::BinExpr {
        lhs: Box::new(Node::Int(2)),
        op: lexer::Token::Mul,
        rhs: Box::new(Node::Int(3)),
      }),
    }),
  );

  test_parse(
    "1 * 2 + 3",
    &parse_bin_expr,
    Ok(Node::BinExpr {
      lhs: Box::new(Node::BinExpr {
        lhs: Box::new(Node::Int(1)),
        op: lexer::Token::Mul,
        rhs: Box::new(Node::Int(2)),
      }),
      op: lexer::Token::Add,
      rhs: Box::new(Node::Int(3)),
    }),
  );
}

#[test]
fn test_fn_expr() {
  test_parse(
    "|| 5",
    &parse_il_expr,
    Ok(Node::FuncDef {
      params: Vec::new(),
      body: Box::new(Node::Return(Some(Box::new(Node::Int(5))))),
    }),
  );

  test_parse(
    "|x| 5",
    &parse_il_expr,
    Ok(Node::FuncDef {
      params: vec![String::from("x")],
      body: Box::new(Node::Return(Some(Box::new(Node::Int(5))))),
    }),
  );

  test_parse(
    "|x,| 5",
    &parse_il_expr,
    Ok(Node::FuncDef {
      params: vec![String::from("x")],
      body: Box::new(Node::Return(Some(Box::new(Node::Int(5))))),
    }),
  );

  test_parse(
    "|x,y| 5",
    &parse_il_expr,
    Ok(Node::FuncDef {
      params: vec![String::from("x"), String::from("y")],
      body: Box::new(Node::Return(Some(Box::new(Node::Int(5))))),
    }),
  );
}

#[test]
fn test_decl() {
  test_parse("x", &parse_decl, Ok(Var::Single(String::from("x"))));

  test_parse(
    "[x]",
    &parse_decl,
    Ok(Var::Multi(vec![Var::Single(String::from("x"))])),
  );

  test_parse(
    "[x, y]",
    &parse_decl,
    Ok(Var::Multi(vec![
      Var::Single(String::from("x")),
      Var::Single(String::from("y")),
    ])),
  );

  test_parse(
    "[[x, y], z]",
    &parse_decl,
    Ok(Var::Multi(vec![
      Var::Multi(vec![
        Var::Single(String::from("x")),
        Var::Single(String::from("y")),
      ]),
      Var::Single(String::from("z")),
    ])),
  );

  test_parse(
    "[x, [y, z]]",
    &parse_decl,
    Ok(Var::Multi(vec![
      Var::Single(String::from("x")),
      Var::Multi(vec![
        Var::Single(String::from("y")),
        Var::Single(String::from("z")),
      ]),
    ])),
  );

  test_parse(
    "[[x], [y]]",
    &parse_decl,
    Ok(Var::Multi(vec![
      Var::Multi(vec![Var::Single(String::from("x"))]),
      Var::Multi(vec![Var::Single(String::from("y"))]),
    ])),
  );

  test_parse(
    "[x, [y, z], q]",
    &parse_decl,
    Ok(Var::Multi(vec![
      Var::Single(String::from("x")),
      Var::Multi(vec![
        Var::Single(String::from("y")),
        Var::Single(String::from("z")),
      ]),
      Var::Single(String::from("q")),
    ])),
  );
}

#[test]
fn test_return_stmt() {
  test_parse("return", &parse_stmt, Ok(Node::Return(None)));

  test_parse(
    "return 5",
    &parse_stmt,
    Ok(Node::Return(Some(Box::new(Node::Int(5))))),
  );

  test_parse(
    "return fn()
       return 5",
    &parse_stmt,
    Ok(Node::Return(Some(Box::new(Node::FuncDef {
      params: Vec::new(),
      body: Box::new(Node::Block(vec![
        Node::Return(Some(Box::new(Node::Int(5)))),
      ])),
    })))),
  );
}

#[test]
fn test_if_stmt() {
  test_parse(
    "if true
       pass",
    &parse_stmt,
    Ok(Node::If {
      cond: Box::new(Node::Bool(true)),
      body: Box::new(Node::Block(vec![Node::Pass])),
      els: None,
    }),
  );

  test_parse(
    "else if true
       pass",
    &parse_stmt,
    Ok(Node::ElseIf {
      cond: Box::new(Node::Bool(true)),
      body: Box::new(Node::Block(vec![Node::Pass])),
    }),
  );

  test_parse(
    "else
       pass",
    &parse_stmt,
    Ok(Node::Else {
      body: Box::new(Node::Block(vec![Node::Pass])),
    }),
  );
}

#[test]
fn test_for_stmt() {
  test_parse(
    "for x in true
       pass",
    &parse_stmt,
    Ok(Node::For {
      decl: Var::Single(String::from("x")),
      expr: Box::new(Node::Bool(true)),
      body: Box::new(Node::Block(vec![Node::Pass])),
    }),
  );
}

#[test]
fn test_while_stmt() {
  test_parse(
    "while true
       pass",
    &parse_stmt,
    Ok(Node::While {
      expr: Box::new(Node::Bool(true)),
      body: Box::new(Node::Block(vec![Node::Pass])),
    }),
  );
}

#[test]
fn test_loop_stmt() {
  test_parse(
    "loop
       pass",
    &parse_stmt,
    Ok(Node::Loop {
      body: Box::new(Node::Block(vec![Node::Pass])),
    }),
  );
}

#[test]
fn test_place() {
  test_parse(
    "x",
    &parse_place,
    Ok(Place::Single(Box::new(Node::Name(String::from("x"))))),
  );

  test_parse(
    "[x]",
    &parse_place,
    Ok(Place::Multi(vec![
      Place::Single(Box::new(Node::Name(String::from("x")))),
    ])),
  );

  test_parse(
    "[x.y]",
    &parse_place,
    Ok(Place::Multi(vec![
      Place::Single(Box::new(Node::Index {
        lhs: Box::new(Node::Name(String::from("x"))),
        rhs: Box::new(Node::Str(String::from("y"))),
      })),
    ])),
  );

  test_parse(
    "[x, y]",
    &parse_place,
    Ok(Place::Multi(vec![
      Place::Single(Box::new(Node::Name(String::from("x")))),
      Place::Single(Box::new(Node::Name(String::from("y")))),
    ])),
  );

  test_parse(
    "[[x, y], z]",
    &parse_place,
    Ok(Place::Multi(vec![
      Place::Multi(vec![
        Place::Single(Box::new(Node::Name(String::from("x")))),
        Place::Single(Box::new(Node::Name(String::from("y")))),
      ]),
      Place::Single(Box::new(Node::Name(String::from("z")))),
    ])),
  );

  test_parse(
    "[x, [y, z]]",
    &parse_place,
    Ok(Place::Multi(vec![
      Place::Single(Box::new(Node::Name(String::from("x")))),
      Place::Multi(vec![
        Place::Single(Box::new(Node::Name(String::from("y")))),
        Place::Single(Box::new(Node::Name(String::from("z")))),
      ]),
    ])),
  );

  test_parse(
    "[[x], [y]]",
    &parse_place,
    Ok(Place::Multi(vec![
      Place::Multi(vec![Place::Single(Box::new(Node::Name(String::from("x"))))]),
      Place::Multi(vec![Place::Single(Box::new(Node::Name(String::from("y"))))]),
    ])),
  );

  test_parse(
    "[x, [y, z], q]",
    &parse_place,
    Ok(Place::Multi(vec![
      Place::Single(Box::new(Node::Name(String::from("x")))),
      Place::Multi(vec![
        Place::Single(Box::new(Node::Name(String::from("y")))),
        Place::Single(Box::new(Node::Name(String::from("z")))),
      ]),
      Place::Single(Box::new(Node::Name(String::from("q")))),
    ])),
  );
}
