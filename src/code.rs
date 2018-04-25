

#[derive(Debug,PartialEq,Clone)]
pub enum Data {
  Null,
  Int(i64),
  Float(f64),
  Bool(bool),
  Str(String),
  Func, // FIXME
  Table, // FIXME
}

#[derive(Debug,PartialEq,Clone)]
pub enum Instr {
  Push(Data),
  Pop,
  Dup,
}
