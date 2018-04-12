use super::*;
use super::super::lexer;
use codemap::CodeMap;
use codemap::Spanned;
use std::fmt::Debug;
use std::cmp::PartialEq;

fn get_tokens(source: &str) -> Vec<Spanned<Token>> {
  let mut map = CodeMap::new();
  let file = map.add_file(String::from("_test"), String::from(source));
  lexer::lex(&file)
}

fn test_parse<T: Debug + PartialEq>(source: &str, func: &Fn(&mut ParseIter) -> T, expect: T) {
  let tokens = get_tokens(source);
  let mut it = tokens.iter().peekable();

  assert_eq!(func(&mut it), expect);

  // wait, why do these all use parse_un_expr instead of the &Fn we get?
  assert_eq!(
    parse_un_expr(&mut it),
    Err(UnexpectedToken(lexer::Token::End))
  );
  it.next();

  assert_eq!(
    parse_un_expr(&mut it),
    Err(UnexpectedToken(lexer::Token::EOF))
  );
  it.next();

  assert_eq!(parse_un_expr(&mut it), Err(UnexpectedEOF));
}

#[test]
fn test_quark() {
  test_parse("null", &parse_quark, Ok(Node::Null));
  test_parse("true", &parse_quark, Ok(Node::Bool(true)));
  test_parse("false", &parse_quark, Ok(Node::Bool(false)));
  test_parse("1.3", &parse_quark, Ok(Node::Float(1.3)));
  test_parse("0.3", &parse_quark, Ok(Node::Float(0.3)));
  test_parse("2", &parse_quark, Ok(Node::Int(2)));
  test_parse("3", &parse_quark, Ok(Node::Int(3)));
  test_parse("name", &parse_quark, Ok(Node::Name(String::from("name"))));
  test_parse("table", &parse_quark, Ok(Node::Table));
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
    Ok(Node::Call {
      func: Box::new(Node::Name(String::from("foo"))),
      args: Vec::new(),
    }),
  );
  test_parse(
    "foo:bar()",
    &parse_simple,
    Ok(Node::Method {
      owner: Box::new(Node::Name(String::from("foo"))),
      method: Box::new(Node::Str(String::from("bar"))),
      args: Vec::new(),
    }),
  );
  test_parse(
    "foo.bar()",
    &parse_simple,
    Ok(Node::Call {
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
    Ok(Node::Method {
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
fn test_fn_args() {
  test_parse("()", &parse_fn_args, Ok(Vec::new()));
}

#[test]
fn test_un_expr() {
  test_parse("5", &parse_un_expr, Ok(Node::Int(5)));

  test_parse(
    "foo()",
    &parse_un_expr,
    Ok(Node::Call {
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
}

#[test]
fn test_fn_expr() {
  test_parse(
    "|| 5",
    &parse_il_expr,
    Ok(Node::Lambda {
      params: Vec::new(),
      expr: Box::new(Node::Int(5)),
    }),
  );

  test_parse(
    "|x| 5",
    &parse_il_expr,
    Ok(Node::Lambda {
      params: vec![String::from("x")],
      expr: Box::new(Node::Int(5)),
    }),
  );

  test_parse(
    "|x,| 5",
    &parse_il_expr,
    Ok(Node::Lambda {
      params: vec![String::from("x")],
      expr: Box::new(Node::Int(5)),
    }),
  );

  test_parse(
    "|x,y| 5",
    &parse_il_expr,
    Ok(Node::Lambda {
      params: vec![String::from("x"), String::from("y")],
      expr: Box::new(Node::Int(5)),
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
