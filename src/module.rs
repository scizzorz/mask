use bincode::serialize;
use bincode;
use blake2::Blake2b;
// use blake2::Digest;
use blake2::digest::Input;
use blake2::digest::VariableOutput;
use code::Data;
use code::Instr;
use codemap::CodeMap;
use codemap::File;
use compiler::CompileErrorKind;
use compiler::Compiler;
use lexer;
use parser::ParseErrorKind;
use parser;
use semck::CheckErrorKind;
use semck::SemChecker;
use std::fs;
use std::io::Read;
use std::io;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug)]
pub enum ModuleErrorKind {
  CheckError(CheckErrorKind),
  ParseError(ParseErrorKind),
  CompileError(CompileErrorKind),
  IOError(io::Error),
  BincodeError(bincode::Error),
}

fn hash_bytes(bytes: Vec<u8>) -> [u8; 8] {
  // these unwraps should be safe because the output size is hardcoded
  let mut hasher = Blake2b::new(8).unwrap();
  hasher.process(&bytes);
  let mut buf = [0u8; 8];
  hasher.variable_result(&mut buf).unwrap();
  buf
}

#[derive(Serialize, Deserialize)]
pub struct Module {
  lex_hash: [u8; 8],
  ast_hash: [u8; 8],
  pub code: Vec<Instr>,
  pub consts: Vec<Data>,
}

impl Module {
  pub fn from_string(map: &mut CodeMap, chunk: &str) -> Result<Module, ModuleErrorKind> {
    let file = map.add_file(String::from("_anon"), chunk.to_string());
    Module::new(map, file)
  }

  pub fn from_file(map: &mut CodeMap, filename: &str) -> Result<Module, ModuleErrorKind> {
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

  pub fn new(map: &CodeMap, file: Arc<File>) -> Result<Module, ModuleErrorKind> {
    let cache_filename = &format!("{}c", file.name());
    let cache_path = Path::new(&cache_filename);

    // if we can't read the cache, it's not a fatal error,
    // or really even an error worth reporting... (is it?)
    let cached: Option<Module> = match fs::File::open(&cache_path) {
      Ok(file) => {
        bincode::deserialize_from(file).ok()
      }
      Err(_) => None,
    };

    // generate tokens
    let tokens = lexer::lex(&file);

    // hash tokens
    let hashable_tokens: Vec<_> = tokens.iter().map(|x| x.node.clone()).collect();
    let lex_bytes = serialize(&hashable_tokens);
    let lex_hash = match lex_bytes {
      Ok(x) => hash_bytes(x),
      Err(why) => return Err(ModuleErrorKind::BincodeError(why)),
    };

    if let Some(cached) = cached {
      if cached.lex_hash == lex_hash {
        println!("returning from lex cache");
        return Ok(cached);
      }
    }

    // generate AST
    let mut ast = match parser::parse(tokens) {
      Ok(root) => root,
      Err(why) => return Err(ModuleErrorKind::ParseError(why)),
    };

    // rearrange AST
    let mut ck = SemChecker::new();
    match ck.check(&mut ast) {
      Err(why) => return Err(ModuleErrorKind::CheckError(why)),
      _ => {}
    }

    // hash AST
    let ast_bytes = serialize(&ast);
    let ast_hash = match ast_bytes {
      Ok(x) => hash_bytes(x),
      Err(why) => return Err(ModuleErrorKind::BincodeError(why)),
    };

    /*
    if let Some(cached) = cached {
      if cached.ast_hash == ast_hash {
        println!("returning from ast cache");
        return Ok(cached);
      }
    }
    */

    // generate bytecode
    let mut compiler = Compiler::new();
    match compiler.compile(&ast) {
      Err(why) => return Err(ModuleErrorKind::CompileError(why)),
      _ => {}
    }

    let ret = Module {
      lex_hash,
      ast_hash,
      code: compiler.get_instrs(),
      consts: compiler.get_consts(),
    };

    // again, if we can'write read the cache, it's not a fatal error.
    match fs::File::create(&cache_path) {
      Ok(file) => {
        bincode::serialize_into(file, &ret);
      }
      Err(_) => {},
    };

    Ok(ret)
  }
}
