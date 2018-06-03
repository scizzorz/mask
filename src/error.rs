use bincode;
use lexer::Token;
use std::io;

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
  MissingCurrentBlock,
}

#[derive(Debug)]
pub enum ModuleErrorKind {
  CheckError(CheckErrorKind),
  ParseError(ParseErrorKind),
  CompileError(CompileErrorKind),
  IOError(io::Error),
  BincodeError(bincode::Error),
}

#[derive(Debug)]
pub enum EngineErrorKind {
  ModuleError(ModuleErrorKind),
  ExecuteError(ExecuteControl),
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
