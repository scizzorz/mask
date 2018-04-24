use codemap::CodeMap;
use codemap::File;
use module::Module;

pub struct Engine<'a> {
  pub map: CodeMap,
  pub mods: Vec<Module<'a>>,
}
