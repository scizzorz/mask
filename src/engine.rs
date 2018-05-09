use code::Const;
use code::Data;
use code::Instr;
use code::Item;
use codemap::CodeMap;
use float;
use float_base;
use int;
use lexer::Token;
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
  BadBinOp(Token),
  BadUnOp(Token),
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

      Instr::BinOp(ref op) => {
        // this should guarantee that we can pop/unwrap twice
        if self.data_stack.len() < 2 {
          return Err(ExecuteErrorKind::EmptyStack);
        }

        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();

        match (&rhs.val, &lhs.val) {
          (&Data::Int(x), &Data::Int(y)) => {
            let data = Engine::ex_bin_int(op, x, y)?;
            self.data_stack.push(Data::Int(data).to_item());
          }
          (&Data::Int(x), &Data::Float(y)) => {
            let data = Engine::ex_bin_float(op, float::from(x as float_base), y)?;
            self.data_stack.push(Data::Float(data).to_item());
          }
          (&Data::Float(x), &Data::Int(y)) => {
            let data = Engine::ex_bin_float(op, x, float::from(y as float_base))?;
            self.data_stack.push(Data::Float(data).to_item());
          }
          (&Data::Float(x), &Data::Float(y)) => {
            let data = Engine::ex_bin_float(op, x, y)?;
            self.data_stack.push(Data::Float(data).to_item());
          }
          _ => {}
        }
      }

      Instr::UnOp(ref op) => {}

      _ => {
        println!("WARNING: Unable to use instruction: {:?}", instr);
      }
    }

    Ok(())
  }

  fn ex_bin_int(op: &Token, x: int, y: int) -> Result<int, ExecuteErrorKind> {
    match *op {
      Token::Add => Ok(x + y),
      Token::Sub => Ok(x - y),
      Token::Mul => Ok(x * y),
      Token::Div => Ok(x / y),
      _ => Err(ExecuteErrorKind::BadBinOp(op.clone())),
    }
  }

  fn ex_bin_float(op: &Token, x: float, y: float) -> Result<float, ExecuteErrorKind> {
    let x = x.into_inner();
    let y = y.into_inner();
    match *op {
      Token::Add => Ok(float::from(x + y)),
      Token::Sub => Ok(float::from(x - y)),
      Token::Mul => Ok(float::from(x * y)),
      Token::Div => Ok(float::from(x / y)),
      _ => Err(ExecuteErrorKind::BadBinOp(op.clone())),
    }
  }
}
