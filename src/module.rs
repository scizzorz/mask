use codemap::CodeMap;
use codemap::File;
use lexer;
use parser::Node;
use parser::ParseErrorKind;
use parser;
use semck::CheckErrorKind;
use semck;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::io;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug)]
pub enum ModuleErrorKind {
  CheckError(CheckErrorKind),
  ParseError(ParseErrorKind),
  IOError(io::Error),
}

pub struct Module<'a> {
  map: &'a CodeMap,
  file: Arc<File>,
  pub ast: Node,
}

impl<'a> Module<'a> {
  pub fn from_string(map: &'a mut CodeMap, chunk: &str) -> Result<Module<'a>, ModuleErrorKind> {
    let file = map.add_file(String::from("_anon"), chunk.to_string());
    Module::new(map, file)
  }

  pub fn from_file(map: &'a mut CodeMap, filename: &str) -> Result<Module<'a>, ModuleErrorKind> {
    let path = Path::new(&filename);
    let mut fs_file = match fs::File::open(path) {
      Ok(file) => file,
      Err(why) => return Err(ModuleErrorKind::IOError(why)),
    };

    let mut contents = String::new();
    let file = match fs_file.read_to_string(&mut contents) {
      Ok(_) => map.add_file(filename.to_string(), contents.to_string()),
      Err(why) => return Err(ModuleErrorKind::IOError(why)),
    };

    Module::new(map, file)
  }

  pub fn new(map: &'a CodeMap, file: Arc<File>) -> Result<Module<'a>, ModuleErrorKind> {
    let tokens = lexer::lex(&file);
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
