use lexer::Token;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Instr {
  Block(Vec<Instr>),
  Dup,
  If(Vec<Instr>),
  IfElse(Vec<Instr>, Vec<Instr>),
  Jump(isize),
  Nop,
  Pop,
  Print,
  PushConst(usize),
  PushScope,
  NewTable,
  Set,
  Get,
  BinOp(Token),
  UnOp(Token),
  CmpOp(Token, bool),
  LogicOp(Token),
  Returnable(Vec<Instr>),
  Truth,
}
