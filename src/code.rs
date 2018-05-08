use float;
use int;
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
  Func, // FIXME
  Table(HashMap<Data, Item>),
}

impl Data {
  pub fn truth(&self) -> bool {
    match *self {
      Data::Null | Data::Bool(false) => false,
      _ => true,
    }
  }

  pub fn to_const(&self) -> Const {
    match *self {
      Data::Int(x) => Const::Int(x),
      Data::Float(x) => Const::Float(x),
      Data::Bool(x) => Const::Bool(x),
      Data::Str(ref x) => Const::Str(x.clone()),
      _ => Const::Null,
    }
  }

  pub fn set_key(&mut self, key: Data, val: Item) {
    match *self {
      Data::Table(ref mut map) => {
        map.insert(key, val);
      }
      _ => {}
    }
  }

  pub fn get_key(&self, key: &Data) -> Item {
    if let Data::Table(ref map) = *self {
      if let Some(k) = map.get(key) {
        return k.clone();
      }
    }
    (Const::Null).to_item()
  }
}

impl Hash for Data {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match *self {
      Data::Table(ref _x) => {
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

  pub fn to_item(&self) -> Item {
    Item {
      val: self.to_data(),
      meta: Data::Null,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Item {
  pub val: Data,
  pub meta: Data,
}

impl Item {
  pub fn truth(&self) -> bool {
    self.val.truth()
  }
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
