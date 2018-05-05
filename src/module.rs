use VERSION;
use bincode::serialize;
use bincode;
use blake2::Blake2b;
use blake2::digest::Input;
use blake2::digest::VariableOutput;
use code::Const;
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
use std::fs::OpenOptions;
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
  version: [u8; 4],
  file_hash: [u8; 8],
  lex_hash: [u8; 8],
  ast_hash: [u8; 8],
  pub code: Vec<Instr>,
  pub consts: Vec<Const>,
}

impl Module {
  pub fn from_string(map: &mut CodeMap, chunk: &str) -> Result<Module, ModuleErrorKind> {
    let file = map.add_file(String::from("_anon"), chunk.to_string());
    Module::new(file, false)
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

    Module::new(file, true)
  }

  pub fn write_cache(&self, cache_path: &Path) -> Option<()> {
    match OpenOptions::new()
      .write(true)
      .truncate(true)
      .create(true)
      .open(&cache_path)
    {
      Ok(file) => bincode::serialize_into(file, &self).ok(),
      _ => None,
    }
  }

  pub fn read_cache(cache_path: &Path) -> Option<Module> {
    match fs::File::open(&cache_path) {
      Ok(file) => bincode::deserialize_from(file).ok(),
      _ => None,
    }
  }

  fn new(file: Arc<File>, use_cache: bool) -> Result<Module, ModuleErrorKind> {
    let cache_filename = &format!("{}c", file.name());
    let cache_path = Path::new(&cache_filename);

    // if we can't read the cache, it's not a fatal error,
    // or really even an error worth reporting... (is it?)
    let cache = match (use_cache, Module::read_cache(&cache_path)) {
      (true, Some(cache)) => match cache.version {
        VERSION => Some(cache),
        _ => None,
      },
      _ => None,
    };

    // copy the hash values out so we can match on them instead of cache
    // (can't figure out move semantics to match on something multiple times
    // and possibly return it each time)
    let (file_cache, lex_cache, ast_cache) = match cache {
      Some(ref cache) => (
        Some(cache.file_hash),
        Some(cache.lex_hash),
        Some(cache.ast_hash),
      ),
      _ => (None, None, None),
    };

    // hash file
    let file_hash = hash_bytes(file.source().as_bytes().to_vec());

    // check file cache
    if let Some(file_cache) = file_cache {
      if file_cache == file_hash {
        let ret = cache.unwrap();
        return Ok(ret);
      }
    }

    // generate tokens
    let tokens = lexer::lex(&file);

    // hash tokens
    let hashable_tokens: Vec<_> = tokens.iter().map(|x| x.node.clone()).collect();
    let lex_bytes = serialize(&hashable_tokens);
    let lex_hash = match lex_bytes {
      Ok(x) => hash_bytes(x),
      Err(why) => return Err(ModuleErrorKind::BincodeError(why)),
    };

    // check lex cache
    if let Some(lex_cache) = lex_cache {
      if lex_cache == lex_hash {
        let ret = Module {
          file_hash,
          ..cache.unwrap()
        };
        ret.write_cache(&cache_path);
        return Ok(ret);
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

    // check AST cache
    if let Some(ast_cache) = ast_cache {
      if ast_cache == ast_hash {
        let ret = Module {
          file_hash,
          lex_hash,
          ..cache.unwrap()
        };
        ret.write_cache(&cache_path);
        return Ok(ret);
      }
    }

    // generate bytecode
    let mut compiler = Compiler::new();
    match compiler.compile(&ast) {
      Err(why) => return Err(ModuleErrorKind::CompileError(why)),
      _ => {}
    }

    let ret = Module {
      version: ::VERSION,
      file_hash,
      lex_hash,
      ast_hash,
      code: compiler.block,
      consts: compiler.consts,
    };

    if use_cache {
      // again, if we can't write read the cache, it's not a fatal error.
      ret.write_cache(&cache_path);
    }

    Ok(ret)
  }
}
