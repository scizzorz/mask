use engine::Engine;
use engine::Execute;
use float;
use gc::Gc;
use gc::GcCell;
use int;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::mem;

type Table = Gc<GcCell<HashMap<Data, Item>>>;

#[derive(Clone)]
pub struct RustFunc(pub &'static Fn(&mut Engine) -> Execute);

impl fmt::Debug for RustFunc {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let addr = unsafe { mem::transmute::<_, u128>(self.0) };
    write!(f, "{:x}", addr)
  }
}

impl PartialEq for RustFunc {
  fn eq(&self, other: &RustFunc) -> bool {
    let (self_addr, other_addr) = unsafe {
      (
        mem::transmute::<_, u128>(self.0),
        mem::transmute::<_, u128>(other.0),
      )
    };
    self_addr == other_addr
  }
}

impl Eq for RustFunc {}

#[derive(Debug, PartialEq, Eq, Clone, Trace, Finalize)]
pub enum Data {
  Null,
  Int(int),
  Float(#[unsafe_ignore_trace] float),
  Bool(bool),
  Str(Gc<String>),
  Func {
    id: usize,
    nargs: usize,
    module: String,
  },
  Rust(#[unsafe_ignore_trace] RustFunc),
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

  pub fn null(&self) -> bool {
    match *self {
      Data::Null => true,
      _ => false,
    }
  }

  pub fn to_const(&self) -> Const {
    match *self {
      Data::Int(x) => Const::Int(x),
      Data::Float(x) => Const::Float(x),
      Data::Bool(x) => Const::Bool(x),
      Data::Str(ref x) => Const::Str((**x).clone()),
      _ => Const::Null,
    }
  }

  pub fn can_set_key(&self) -> bool {
    match *self {
      Data::Table(_) => true,
      _ => false,
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
    Const::Null.to_item()
  }

  pub fn to_string(&self) -> String {
    match *self {
      Data::Null => String::from("null"),
      Data::Int(x) => format!("{}", x),
      Data::Float(x) => format!("{}", x),
      Data::Bool(x) => format!("{}", x),
      Data::Str(ref x) => (**x).clone(),
      Data::Func {
        id,
        ref module,
        nargs,
      } => format!("func[{}/{} from {}]", id, nargs, module),
      Data::Table(ref x) => {
        let addr = unsafe { mem::transmute::<_, u64>(x.clone()) };
        format!("table[{:x}]", addr)
      }
      Data::Rust(ref x) => format!("rustfunc[{:?}]", x),
    }
  }

  pub fn to_item(&self) -> Item {
    Item {
      val: self.clone(),
      sup: None,
    }
  }

  pub fn into_item(self) -> Item {
    Item {
      val: self,
      sup: None,
    }
  }
}

impl Hash for Data {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match *self {
      Data::Table(ref x) => {
        let addr = unsafe { mem::transmute::<_, u64>(x.clone()) };
        addr.hash(state);
      }
      Data::Func {
        id,
        nargs,
        module: _,
      } => {
        // seems reasonable
        //        m                 a       s k
        ((id + 0x6d) * ((nargs * 0x61) + 0x736b)).hash(state);
      }
      Data::Rust(ref x) => {
        let addr = unsafe { mem::transmute::<_, u128>(x.0) };
        addr.hash(state);
      }
      Data::Null => {
        (0).hash(state); // FIXME?
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
  pub fn into_data(self) -> Data {
    match self {
      Const::Null => Data::Null,
      Const::Int(x) => Data::Int(x),
      Const::Float(x) => Data::Float(x),
      Const::Bool(x) => Data::Bool(x),
      Const::Str(x) => Data::Str(Gc::new(x)),
    }
  }

  pub fn to_data(&self) -> Data {
    match *self {
      Const::Null => Data::Null,
      Const::Int(x) => Data::Int(x),
      Const::Float(x) => Data::Float(x),
      Const::Bool(x) => Data::Bool(x),
      Const::Str(ref x) => Data::Str(Gc::new(x.clone())),
    }
  }

  pub fn to_item(&self) -> Item {
    self.to_data().into_item()
  }

  pub fn into_item(self) -> Item {
    self.into_data().into_item()
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Trace, Finalize)]
pub struct Item {
  pub val: Data,
  pub sup: Option<Box<Item>>,
}

impl Item {
  pub fn truth(&self) -> bool {
    self.val.truth()
  }

  pub fn null(&self) -> bool {
    self.val.null()
  }

  // FIXME something goes wrong here when using a table as a key
  pub fn set_key(&mut self, key: Data, val: Item) {
    if self.val.can_set_key() {
      self.val.set_key(key, val);
    } else if let Some(ref mut bx) = self.sup {
      bx.set_key(key, val);
    }
  }

  pub fn get_key(&self, key: &Data) -> Item {
    if self.val.contains_key(key) {
      return self.val.get_key(key);
    }

    if let Some(ref bx) = self.sup {
      return bx.get_key(key);
    }

    Const::Null.to_item()
  }

  pub fn to_string(&self) -> String {
    self.val.to_string()
  }
}
