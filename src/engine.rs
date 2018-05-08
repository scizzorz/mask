use code::Data;
use code::Instr;
use code::Item;
use codemap::CodeMap;
use module::Module;
use module::ModuleErrorKind;
use std::collections::HashMap;

pub struct Engine {
  pub map: CodeMap,
  pub mods: Vec<Module>,
  data_stack: Vec<Item>,
  scope: Item,
}

#[derive(Debug)]
pub enum EngineErrorKind {
  ModuleError(ModuleErrorKind),
}

impl Engine {
  pub fn new() -> Engine {
    Engine {
      map: CodeMap::new(),
      mods: Vec::new(),
      data_stack: Vec::new(),
      scope: Item {
        val: Data::Table(HashMap::new()),
        meta: Data::Null,
      },
    }
  }

  pub fn import(&mut self, filename: &str) -> Result<(()), EngineErrorKind> {
    let module = match Module::from_file(&mut self.map, filename) {
      Err(why) => return Err(EngineErrorKind::ModuleError(why)),
      Ok(module) => module,
    };

    self.ex_many(&module, &module.code);

    self.mods.push(module);

    Ok(())
  }

  fn ex_many(&mut self, module: &Module, instrs: &Vec<Instr>) {
    for instr in instrs {
      self.ex(module, instr);
    }
  }

  fn ex(&mut self, module: &Module, instr: &Instr) {
    match *instr {
      Instr::PushConst(x) => {
        self.data_stack.push(module.consts[x].to_item());
      }
      Instr::Pop => {
        self.data_stack.pop();
      }
      Instr::Dup => match self.data_stack.pop() {
        Some(x) => {
          self.data_stack.push(x.clone());
          self.data_stack.push(x);
        }
        None => println!("WARNING: attempting to dup empty stack"),
      },
      Instr::Nop => {}
      Instr::Print => match self.data_stack.pop() {
        Some(x) => println!("{:?}", x),
        None => println!("WARNING: attempting to print empty stack"),
      },

      Instr::If(ref body) => match self.data_stack.pop() {
        Some(x) => {
          if x.truth() {
            self.ex_many(module, body);
          }
        }
        None => println!("WARNING: attempting to if empty stack"),
      },

      _ => {
        println!("WARNING: Unable to use instruction: {:?}", instr);
      }
    }
  }
}
