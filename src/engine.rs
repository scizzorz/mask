use code::Const;
use code::Data;
use code::Instr;
use code::Item;
use codemap::CodeMap;
use module::Module;
use module::ModuleErrorKind;

struct RuntimeModule {
  pub code: Vec<Instr>,
  pub consts: Vec<Const>,
  pub scope: Item,
}

impl RuntimeModule {
  fn from_static(module: Module) -> RuntimeModule {
    RuntimeModule {
      code: module.code,
      consts: module.consts,
      scope: Item {
        val: Data::new_table(),
        meta: None,
      },
    }
  }
}

pub struct Engine {
  pub map: CodeMap,
  mods: Vec<RuntimeModule>,
  data_stack: Vec<Item>,
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
    }
  }

  pub fn import(&mut self, filename: &str) -> Result<(()), EngineErrorKind> {
    let module = match Module::from_file(&mut self.map, filename) {
      Err(why) => return Err(EngineErrorKind::ModuleError(why)),
      Ok(module) => RuntimeModule::from_static(module),
    };

    self.ex_many(&module, &module.code);

    self.mods.push(module);

    Ok(())
  }

  fn ex_many(&mut self, module: &RuntimeModule, instrs: &Vec<Instr>) {
    for instr in instrs {
      self.ex(module, instr);
    }
  }

  fn ex(&mut self, module: &RuntimeModule, instr: &Instr) {
    println!("executing {:?}", instr);
    match *instr {
      Instr::PushConst(x) => {
        self.data_stack.push(module.consts[x].to_item());
      }
      Instr::PushScope => {
        self.data_stack.push(module.scope.clone());
      }
      Instr::Set => {
        // FIXME
        let mut scope = self.data_stack.pop().unwrap();
        let key = self.data_stack.pop().unwrap().val.clone();
        let val = self.data_stack.pop().unwrap();
        println!("setting key! [{:?}] = {:?}", key, val);
        scope.set_key(key, val);
      }
      Instr::Get => {
        // FIXME
        let scope = self.data_stack.pop().unwrap();
        let key = self.data_stack.pop().unwrap();
        let val = scope.get_key(&key.val);
        self.data_stack.push(val);
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
