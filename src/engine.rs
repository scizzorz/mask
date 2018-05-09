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
pub enum ExecuteErrorKind {
  Exception(Item),
  EmptyStack,
}

#[derive(Debug)]
pub enum EngineErrorKind {
  ModuleError(ModuleErrorKind),
  ExecuteError(ExecuteErrorKind),
}

type Execute = Result<(), ExecuteErrorKind>;

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

    match self.ex_many(&module, &module.code) {
      Err(why) => return Err(EngineErrorKind::ExecuteError(why)),
      Ok(_) => {}
    }

    self.mods.push(module);

    Ok(())
  }

  fn ex_many(&mut self, module: &RuntimeModule, instrs: &Vec<Instr>) -> Execute {
    for instr in instrs {
      self.ex(module, instr)?;
    }
    Ok(())
  }

  fn ex(&mut self, module: &RuntimeModule, instr: &Instr) -> Execute {
    match *instr {
      Instr::PushConst(x) => {
        self.data_stack.push(module.consts[x].to_item());
      }

      Instr::PushScope => {
        self.data_stack.push(module.scope.clone());
      }

      Instr::NewTable => {
        self.data_stack.push(Item {
          val: Data::new_table(),
          meta: None,
        });
      }

      Instr::Set => {
        // this should guarantee that we can pop/unwrap thrice
        if self.data_stack.len() < 3 {
          return Err(ExecuteErrorKind::EmptyStack);
        }

        let mut scope = self.data_stack.pop().unwrap();
        let key = self.data_stack.pop().unwrap();
        let val = self.data_stack.pop().unwrap();
        scope.set_key(key.val.clone(), val);
      }

      Instr::Get => {
        // this should guarantee that we can pop/unwrap twice
        if self.data_stack.len() < 2 {
          return Err(ExecuteErrorKind::EmptyStack);
        }

        let scope = self.data_stack.pop().unwrap();
        let key = self.data_stack.pop().unwrap();
        let val = scope.get_key(&key.val);
        self.data_stack.push(val);
      }

      Instr::Pop => match self.data_stack.pop() {
        Some(_) => {}
        None => return Err(ExecuteErrorKind::EmptyStack),
      },

      Instr::Dup => match self.data_stack.pop() {
        Some(x) => {
          self.data_stack.push(x.clone());
          self.data_stack.push(x);
        }
        None => return Err(ExecuteErrorKind::EmptyStack),
      },

      Instr::Nop => {}

      Instr::Print => match self.data_stack.pop() {
        Some(x) => println!("{}", x.to_string()),
        None => return Err(ExecuteErrorKind::EmptyStack),
      },

      Instr::If(ref body) => match self.data_stack.pop() {
        Some(x) => {
          if x.truth() {
            self.ex_many(module, body)?;
          }
        }
        None => return Err(ExecuteErrorKind::EmptyStack),
      },

      _ => {
        println!("WARNING: Unable to use instruction: {:?}", instr);
      }
    }

    Ok(())
  }
}
