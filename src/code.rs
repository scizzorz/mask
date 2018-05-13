use lexer::Token;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Instr {
  BinOp(Token),
  Block(Vec<Instr>),
  Call,
  CmpOp(Token, bool),
  Dup,
  FuncDef(Vec<Instr>),
  Get,
  If(Vec<Instr>),
  IfElse(Vec<Instr>, Vec<Instr>),
  Instr,
  Jump(isize),
  LogicOp(Token),
  NewTable,
  Nop,
  Pop,
  Print,
  PushConst(usize),
  PushScope,
  Return,
  Returnable(Vec<Instr>),
  Set,
  Truth,
  UnOp(Token),
}
