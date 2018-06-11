use lexer::Token;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Instr {
  BinOp(Token),
  Block(Vec<Instr>),
  Break,
  Call(usize),
  Catch(Vec<Instr>),
  CmpOp(Token, bool),
  Continue,
  Dup,
  ForBreak,
  FuncDef(Vec<Instr>),
  Get,
  If(Vec<Instr>),
  IfElse(Vec<Instr>, Vec<Instr>),
  LogicOp(Token),
  Loop(Vec<Instr>),
  MethodGet,
  Nop,
  Pop,
  PushConst(usize),
  PushFunc {
    id: usize, 
    nargs: usize,
  },
  PushScope,
  Return,
  Returnable(Vec<Instr>),
  Set,
  UnOp(Token),
}
