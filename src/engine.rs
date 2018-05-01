use codemap::CodeMap;
use module::Module;

pub struct Engine {
  pub map: CodeMap,
  pub mods: Vec<Module>,
}

impl Engine {
  pub fn new() -> Engine {
    Engine {
      map: CodeMap::new(),
      mods: Vec::new(),
    }
  }
}
