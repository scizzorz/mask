use codemap::CodeMap;
use codemap::File;
use lexer;
use parser::Node;
use parser::ParseErrorKind;
use parser;
use semck::CheckErrorKind;
use semck;

#[derive(Debug, Clone, PartialEq)]
pub enum ModuleErrorKind {
  CheckError(CheckErrorKind),
  ParseError(ParseErrorKind),
}

pub struct Module<'a> {
  map: &'a CodeMap,
  file: &'a File,
  pub ast: Node,
}

impl<'a> Module<'a> {
  pub fn new(map: &'a CodeMap, file: &'a File) -> Result<Module<'a>, ModuleErrorKind> {
    let tokens = lexer::lex(file);
    let mut ast = match parser::parse(tokens) {
      Ok(root) => root,
      Err(why) => return Err(ModuleErrorKind::ParseError(why)),
    };

    let mut ck = semck::SemChecker::new();
    match ck.check(&mut ast) {
      Err(why) => return Err(ModuleErrorKind::CheckError(why)),
      _ => {}
    }

    Ok(Module { map, file, ast })
  }
}
