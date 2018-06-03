use bincode;
use lexer::Token;
use std::io;

#[derive(Debug)]
pub enum ErrorKind {
  Lex(LexErrorKind),
  Parse(ParseErrorKind),
  Check(CheckErrorKind),
  Compile(CompileErrorKind),
  Module(ModuleErrorKind),
  Engine(EngineErrorKind),
  IO(io::Error),
  Bincode(bincode::Error),
  Execute(ExecuteControl),
}

#[derive(Debug)]
pub enum LexErrorKind {
  InvalidChar(char),
  UnclosedStr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
  UnexpectedToken(Token),
  UnexpectedEOF,
  UnknownBinaryOperator,
  UnknownUnaryOperator,
  UnusedPlaces,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckErrorKind {
  NotInLoop,
  MissingIf,
  NotPlace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileErrorKind {
}

#[derive(Debug)]
pub enum ModuleErrorKind {
}

#[derive(Debug)]
pub enum EngineErrorKind {
}

// this is used as a bit of a control flow hack - `Return` and `Exception`
// aren't necessarily errors, but I'm using them with Rust's ? operator
// to skip a lot of boiler plate code later
#[derive(Debug)]
pub enum ExecuteControl {
  Break,
  Continue,
  Exception,
  Return,
  Other,
}
