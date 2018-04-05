use super::*;
use super::super::lexer;
use codemap::CodeMap;
use codemap::Spanned;

fn get_tokens(source: &str) -> Vec<Spanned<Token>> {
  let mut map = CodeMap::new();
  let file = map.add_file(String::from("_test"), String::from(source));
  lexer::lex(&file)
}

#[test]
fn test_quark() {
  let source = "null true false 1.3 0.3 2 3 name table";
  let tokens = get_tokens(source);
  let mut it = tokens.iter().peekable();

  assert_eq!(parse_quark(&mut it), Ok(Node::Null));
  assert_eq!(parse_quark(&mut it), Ok(Node::Bool(true)));
  assert_eq!(parse_quark(&mut it), Ok(Node::Bool(false)));
  assert_eq!(parse_quark(&mut it), Ok(Node::Float(1.3)));
  assert_eq!(parse_quark(&mut it), Ok(Node::Float(0.3)));
  assert_eq!(parse_quark(&mut it), Ok(Node::Int(2)));
  assert_eq!(parse_quark(&mut it), Ok(Node::Int(3)));
  assert_eq!(parse_quark(&mut it), Ok(Node::Name(String::from("name"))));
  assert_eq!(parse_quark(&mut it), Ok(Node::Table));
  assert_eq!(parse_quark(&mut it), Err(UnexpectedToken(lexer::Token::End)));
  it.next();
  assert_eq!(parse_quark(&mut it), Err(UnexpectedToken(lexer::Token::EOF)));
  it.next();
  assert_eq!(parse_quark(&mut it), Err(UnexpectedEOF));
}

#[test]
fn test_atom() {
  let source = "null (null)";
  let tokens = get_tokens(source);
  let mut it = tokens.iter().peekable();

  assert_eq!(parse_atom(&mut it), Ok(Node::Null));
  assert_eq!(parse_atom(&mut it), Ok(Node::Null));
  assert_eq!(parse_atom(&mut it), Err(UnexpectedToken(lexer::Token::End)));
  it.next();
  assert_eq!(parse_atom(&mut it), Err(UnexpectedToken(lexer::Token::EOF)));
  it.next();
  assert_eq!(parse_atom(&mut it), Err(UnexpectedEOF));
}

#[test]
fn test_simple() {
  let source = "foo foo.bar foo[bar] foo() foo:bar() foo.bar() foo.bar[baz]:qux()";
  let tokens = get_tokens(source);
  let mut it = tokens.iter().peekable();

  assert_eq!(parse_simple(&mut it), Ok(Node::Name(String::from("foo"))));
  assert_eq!(parse_simple(&mut it), Ok(Node::Index {
    lhs: Box::new(Node::Name(String::from("foo"))),
    rhs: Box::new(Node::Str(String::from("bar"))),
  }));
  assert_eq!(parse_simple(&mut it), Ok(Node::Index {
    lhs: Box::new(Node::Name(String::from("foo"))),
    rhs: Box::new(Node::Name(String::from("bar"))),
  }));
  assert_eq!(parse_simple(&mut it), Ok(Node::Func {
    func: Box::new(Node::Name(String::from("foo"))),
    args: Vec::new(),
  }));
  assert_eq!(parse_simple(&mut it), Ok(Node::Method {
    owner: Box::new(Node::Name(String::from("foo"))),
    method: Box::new(Node::Str(String::from("bar"))),
    args: Vec::new(),
  }));
  assert_eq!(parse_simple(&mut it), Ok(Node::Func {
    func: Box::new(Node::Index {
      lhs: Box::new(Node::Name(String::from("foo"))),
      rhs: Box::new(Node::Str(String::from("bar"))),
    }),
    args: Vec::new(),
  }));
  assert_eq!(parse_simple(&mut it), Ok(Node::Method {
    owner: Box::new(Node::Index {
      lhs: Box::new(Node::Index {
        lhs: Box::new(Node::Name(String::from("foo"))),
        rhs: Box::new(Node::Str(String::from("bar"))),
      }),
      rhs: Box::new(Node::Name(String::from("baz"))),
    }),
    method: Box::new(Node::Str(String::from("qux"))),
    args: Vec::new(),
  }));
  assert_eq!(parse_simple(&mut it), Err(UnexpectedToken(lexer::Token::End)));
  it.next();
  assert_eq!(parse_simple(&mut it), Err(UnexpectedToken(lexer::Token::EOF)));
  it.next();
  assert_eq!(parse_simple(&mut it), Err(UnexpectedEOF));
}

#[test]
fn test_fn_args() {
  let source = "()";
  let tokens = get_tokens(source);
  let mut it = tokens.iter().peekable();

  assert_eq!(parse_fn_args(&mut it), Ok(Vec::new()));
  assert_eq!(parse_fn_args(&mut it), Err(UnexpectedToken(lexer::Token::End)));
  it.next();
  assert_eq!(parse_fn_args(&mut it), Err(UnexpectedToken(lexer::Token::EOF)));
  it.next();
  assert_eq!(parse_fn_args(&mut it), Err(UnexpectedEOF));
}

#[test]
fn test_un_expr() {
  let source = "5 foo() -5 -foo.bar !-5";
  let tokens = get_tokens(source);
  let mut it = tokens.iter().peekable();

  assert_eq!(parse_un_expr(&mut it), Ok(Node::Int(5)));
  assert_eq!(parse_un_expr(&mut it), Ok(Node::Func {
    func: Box::new(Node::Name(String::from("foo"))),
    args: Vec::new(),
  }));
  assert_eq!(parse_un_expr(&mut it), Ok(Node::UnExpr {
    op: lexer::Token::Sub,
    val: Box::new(Node::Int(5)),
  }));
  assert_eq!(parse_un_expr(&mut it), Ok(Node::UnExpr {
    op: lexer::Token::Sub,
    val: Box::new(Node::Index {
      lhs: Box::new(Node::Name(String::from("foo"))),
      rhs: Box::new(Node::Str(String::from("bar"))),
    }),
  }));
  assert_eq!(parse_un_expr(&mut it), Ok(Node::UnExpr {
    op: lexer::Token::Not,
    val: Box::new(Node::UnExpr {
      op: lexer::Token::Sub,
      val: Box::new(Node::Int(5)),
    }),
  }));
  assert_eq!(parse_un_expr(&mut it), Err(UnexpectedToken(lexer::Token::End)));
  it.next();
  assert_eq!(parse_un_expr(&mut it), Err(UnexpectedToken(lexer::Token::EOF)));
  it.next();
  assert_eq!(parse_un_expr(&mut it), Err(UnexpectedEOF));
}
