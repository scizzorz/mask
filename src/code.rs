#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Data {
  Null,
  Int(i64),
  Float(f64),
  Bool(bool),
  Str(String),
  Func,  // FIXME
  Table, // FIXME
}

impl Data {
  pub fn truth(&self) -> bool {
    match *self {
      Data::Null | Data::Bool(false) => false,
      _ => true,
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Item {
  val: Data,
  meta: Data,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
}
