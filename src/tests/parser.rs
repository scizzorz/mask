use super::*;
use super::super::lexer;
use codemap::CodeMap;

#[test]
fn parse_quarks() {
  let mut map = CodeMap::new();
  let file = map.add_file(String::from("_test"), String::from("null true false 1.3 0.3 2 3 name table"));
  let tokens = lexer::lex(&file);
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
