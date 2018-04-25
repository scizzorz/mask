use codemap::CodeMap;
use module::Module;

pub struct Engine<'a> {
  pub map: CodeMap,
  pub mods: Vec<Module<'a>>,
}

impl<'a> Engine<'a> {
  pub fn new() -> Engine<'a> {
    Engine {
      map: CodeMap::new(),
      mods: Vec::new(),
    }
  }
}
