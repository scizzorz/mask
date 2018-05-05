use ::float;
use ::int;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Data {
  Null,
  Int(int),
  Float(float),
  Bool(bool),
  Str(String),
  Func,  // FIXME
  Table(HashMap<Data, Data>),
}

impl Data {
  pub fn to_const(&self) -> Const {
    match *self {
      Data::Int(x) => Const::Int(x),
      Data::Float(x) => Const::Float(x),
      Data::Bool(x) => Const::Bool(x),
      Data::Str(ref x) => Const::Str(x.clone()),
      _ => Const::Null,
    }
  }
}

impl Hash for Data {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match *self {
      Data::Table(ref x) =>  {
        (0).hash(state); // FIXME LOOOOOL
      }
      _ => {
        self.hash(state);
      }
    }
  }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum Const {
  Null,
  Int(int),
  #[serde(with = "::FloatDef")]
  Float(float),
  Bool(bool),
  Str(String),
}

impl Const {
  pub fn to_data(&self) -> Data {
    match *self {
      Const::Null => Data::Null,
      Const::Int(x) => Data::Int(x),
      Const::Float(x) => Data::Float(x),
      Const::Bool(x) => Data::Bool(x),
      Const::Str(ref x) => Data::Str(x.clone()),
    }
  }
}


impl Data {
  pub fn truth(&self) -> bool {
    match *self {
      Data::Null | Data::Bool(false) => false,
      _ => true,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Item {
  val: Data,
  meta: Data,
}

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
}
