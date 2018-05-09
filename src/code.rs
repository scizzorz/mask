use lexer::Token;
use float;
use gc::Gc;
use gc::GcCell;
use int;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;

type Table = Gc<GcCell<HashMap<Data, Item>>>;

#[derive(Debug, PartialEq, Eq, Clone, Trace, Finalize)]
pub enum Data {
  Null,
  Int(int),
  Float(#[unsafe_ignore_trace] float),
  Bool(bool),
  Str(String),
  Func, // FIXME
  Table(Table),
}

impl Data {
  pub fn new_table() -> Data {
    Data::Table(Gc::new(GcCell::new(HashMap::new())))
  }

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
        map.borrow_mut().insert(key, val);
      }
      _ => {}
    }
  }

  pub fn contains_key(&self, key: &Data) -> bool {
    if let Data::Table(ref map) = *self {
      return map.borrow().contains_key(key);
    }
    false
  }

  pub fn get_key(&self, key: &Data) -> Item {
    if let Data::Table(ref map) = *self {
      if let Some(k) = map.borrow().get(key) {
        return k.clone();
      }
    }
    (Const::Null).to_item()
  }

  pub fn to_string(&self) -> String {
    match *self {
      Data::Null => String::from("null"),
      Data::Int(x) => format!("{}", x),
      Data::Float(x) => format!("{}", x),
      Data::Bool(x) => format!("{}", x),
      Data::Str(ref x) => x.clone(),
      Data::Func => String::from("func"),
      Data::Table(_) => String::from("table"),
    }
  }

  pub fn to_item(self) -> Item {
    Item {
      val: self,
      meta: None,
    }
  }
}

impl Hash for Data {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match *self {
      Data::Table(ref _x) => {
        (0).hash(state); // FIXME LOOOOOL
      }
      Data::Null => {
        (0).hash(state); // FIXME
      }
      Data::Int(x) => {
        x.hash(state);
      }
      Data::Float(x) => {
        x.hash(state);
      }
      Data::Bool(x) => {
        x.hash(state);
      }
      Data::Str(ref x) => {
        x.hash(state);
      }
      _ => {
        (0).hash(state); // FIXME
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
    self.to_data().to_item()
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Trace, Finalize)]
pub struct Item {
  pub val: Data,
  pub meta: Option<Table>,
}

impl Item {
  pub fn truth(&self) -> bool {
    self.val.truth()
  }

  pub fn set_key(&mut self, key: Data, val: Item) {
    self.val.set_key(key, val);
  }

  pub fn get_key(&self, key: &Data) -> Item {
    if self.val.contains_key(key) {
      return self.val.get_key(key);
    }

    (Const::Null).to_item()
  }

  pub fn to_string(&self) -> String {
    self.val.to_string()
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
  PushScope,
  NewTable,
  Set,
  Get,
  BinOp(Token),
  UnOp(Token),
}
