use data::Const;
use data::Data;
use code::Instr;
use data::Item;
use codemap::CodeMap;
use float;
use float_base;
use int;
use lexer::Token;
use module::Module;
use module::ModuleErrorKind;
use serde_yaml;
use std::mem;

struct RuntimeModule {
  pub scope: Item,
}

impl RuntimeModule {
  fn new() -> RuntimeModule {
    RuntimeModule {
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
  funcs: Vec<Instr>,
  data_stack: Vec<Item>,
}

// this is used as a bit of a control flow hack - `Return` and `Exception`
// aren't necessarily errors, but I'm using them with Rust's ? operator
// to skip a lot of boiler plate code later
#[derive(Debug)]
pub enum ExecuteErrorKind {
  BadBinOp(Token),
  BadBinOperands,
  BadCmpOp(Token),
  BadCmpOperands,
  BadLogicOp(Token),
  BadUnOp(Token),
  BadUnOperand,
  Break,
  Continue,
  EmptyStack,
  Exception,
  NotCallable,
  Return,
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
      funcs: Vec::new(),
      mods: Vec::new(),
      data_stack: Vec::new(),
    }
  }

  pub fn import(&mut self, filename: &str) -> Result<(()), EngineErrorKind> {
    let module = match Module::from_file(&mut self.map, filename) {
      Err(why) => return Err(EngineErrorKind::ModuleError(why)),
      Ok(module) => module,
    };

    let mut runtime = RuntimeModule::new();

    println!("YAML: {}", serde_yaml::to_string(&module).unwrap());

    match self.ex_many(&module, &mut runtime, &module.code) {
      Err(why) => return Err(EngineErrorKind::ExecuteError(why)),
      Ok(_) => {}
    }

    //self.mods.push(module);

    Ok(())
  }

  fn ex_many(
    &mut self,
    module: &Module,
    runtime: &mut RuntimeModule,
    instrs: &Vec<Instr>,
  ) -> Execute {
    for instr in instrs {
      self.ex(module, runtime, instr)?;
    }
    Ok(())
  }

  fn ex(&mut self, module: &Module, runtime: &mut RuntimeModule, instr: &Instr) -> Execute {
    // println!("executing {:?} on {:?}", instr, self.data_stack);
    match *instr {
      Instr::PushConst(x) => {
        self.data_stack.push(module.consts[x].to_item());
      }

      Instr::PushScope => {
        self.data_stack.push(runtime.scope.clone());
      }

      Instr::FuncDef(ref body) => {
        // FIXME someway to prevent pushing the same function twice
        // eg functions inside functions
        self.funcs.push(Instr::Returnable(body.clone()));
        self.data_stack.push(Item {
          val: Data::Func(self.funcs.len() - 1),
          meta: Some(Box::new(Item {
            val: Data::new_table(),
            meta: Some(Box::new(runtime.scope.clone())),
          })),
        });
      }

      // FIXME cloning this func can't be right, but how
      // else can self be mutable to add more funcs while
      // self.funcs[x] is immutable?
      Instr::Call => match self.data_stack.pop() {
        None => return Err(ExecuteErrorKind::EmptyStack),
        Some(func) => match func {
          Item {
            val: Data::Func(val),
            meta: Some(ref meta),
          } => {
            let mut new_scope = Item {
              val: Data::new_table(),
              meta: Some(meta.clone()),
            };
            mem::swap(&mut new_scope, &mut runtime.scope);
            let func = self.funcs[val].clone();
            self.ex(module, runtime, &func)?;
            mem::swap(&mut new_scope, &mut runtime.scope);
          }
          Item {
            val: Data::Func(val),
            meta: None,
          } => {
            let func = self.funcs[val].clone();
            self.ex(module, runtime, &func)?;
          }
          _ => return Err(ExecuteErrorKind::NotCallable),
        },
      },

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

      Instr::Truth => match self.data_stack.pop() {
        Some(x) => {
          self.data_stack.push(Data::Bool(x.truth()).to_item());
        }
        None => return Err(ExecuteErrorKind::EmptyStack),
      },

      Instr::If(ref body) => match self.data_stack.pop() {
        Some(x) => {
          if x.truth() {
            self.ex_many(module, runtime, body)?;
          }
        }
        None => return Err(ExecuteErrorKind::EmptyStack),
      },

      Instr::While(ref expr, ref body) => {
        loop {
          self.ex_many(module, runtime, expr)?;
          match self.data_stack.pop() {
            Some(x) => {
              match x.truth() {
                true => {
                  match self.ex_many(module, runtime, body) {
                    Ok(_) => {}
                    Err(ExecuteErrorKind::Break) => break,
                    Err(ExecuteErrorKind::Continue) => continue,
                    err => return err,
                  }
                }
                false => break,
              }
            }
            None => return Err(ExecuteErrorKind::EmptyStack),
          }
        }
      },

      Instr::Loop(ref body) => {
        loop {
          match self.ex_many(module, runtime, body) {
            Ok(_) => {}
            Err(ExecuteErrorKind::Break) => break,
            Err(ExecuteErrorKind::Continue) => continue,
            err => return err,
          }
        }
      },

      Instr::Returnable(ref body) => match self.ex_many(module, runtime, body) {
        Ok(_) => {}
        Err(ExecuteErrorKind::Return) => {}
        err => return err,
      },

      Instr::BinOp(ref op) => {
        // this should guarantee that we can pop/unwrap twice
        if self.data_stack.len() < 2 {
          return Err(ExecuteErrorKind::EmptyStack);
        }

        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();

        if *op == Token::Meta {
          let ret = Item {
            meta: Some(Box::new(rhs)),
            val: lhs.val.clone(),
          };
          self.data_stack.push(ret);
          return Ok(());
        }

        match (&lhs.val, &rhs.val) {
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
          _ => return Err(ExecuteErrorKind::BadBinOperands),
        }
      }

      Instr::UnOp(ref op) => match (self.data_stack.pop(), op) {
        (Some(val), &Token::Mul) => match val.meta {
          Some(ref meta) => {
            self.data_stack.push(*meta.clone());
          }
          None => {
            self.data_stack.push(Const::Null.to_item());
          }
        },
        (Some(_), op) => return Err(ExecuteErrorKind::BadUnOp(op.clone())),
        (None, _) => return Err(ExecuteErrorKind::EmptyStack),
      },

      Instr::LogicOp(ref op) => match (self.data_stack.pop(), op) {
        (Some(val), &Token::And) => {
          if !val.truth() {
            self.data_stack.push(val);
            return Err(ExecuteErrorKind::Return);
          }
        }
        (Some(val), &Token::Or) => {
          if val.truth() {
            self.data_stack.push(val);
            return Err(ExecuteErrorKind::Return);
          }
        }
        (Some(_), op) => return Err(ExecuteErrorKind::BadLogicOp(op.clone())),
        (None, _) => return Err(ExecuteErrorKind::EmptyStack),
      },

      Instr::Return => {
        return Err(ExecuteErrorKind::Return);
      }

      Instr::Break => {
        return Err(ExecuteErrorKind::Break);
      }

      Instr::Continue => {
        return Err(ExecuteErrorKind::Continue);
      }

      Instr::CmpOp(ref op, chain) => {
        // this should guarantee that we can pop/unwrap twice
        if self.data_stack.len() < 2 {
          return Err(ExecuteErrorKind::EmptyStack);
        }

        let rhs = self.data_stack.pop().unwrap();
        let lhs = self.data_stack.pop().unwrap();

        let result = match (&lhs.val, &rhs.val) {
          (&Data::Int(x), &Data::Int(y)) => Engine::ex_cmp_int(op, x, y)?,
          (&Data::Int(x), &Data::Float(y)) => {
            Engine::ex_cmp_float(op, float::from(x as float_base), y)?
          }
          (&Data::Float(x), &Data::Int(y)) => {
            Engine::ex_cmp_float(op, x, float::from(y as float_base))?
          }
          (&Data::Float(x), &Data::Float(y)) => Engine::ex_cmp_float(op, x, y)?,
          (&Data::Bool(x), &Data::Bool(y)) => Engine::ex_cmp_bool(op, x, y)?,
          _ => return Err(ExecuteErrorKind::BadCmpOperands),
        };

        match (chain, result) {
          (true, false) => {
            self.data_stack.push(Data::Bool(result).to_item());
            return Err(ExecuteErrorKind::Return);
          }
          (true, true) => {
            self.data_stack.push(rhs);
          }
          (false, _) => {
            self.data_stack.push(Data::Bool(result).to_item());
          }
        }
      }

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

  fn ex_cmp_int(op: &Token, x: int, y: int) -> Result<bool, ExecuteErrorKind> {
    match *op {
      Token::Lt => Ok(x < y),
      Token::Le => Ok(x <= y),
      Token::Gt => Ok(x > y),
      Token::Ge => Ok(x >= y),
      Token::Eql => Ok(x == y),
      Token::Ne => Ok(x != y),
      _ => Err(ExecuteErrorKind::BadCmpOp(op.clone())),
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

  fn ex_cmp_float(op: &Token, x: float, y: float) -> Result<bool, ExecuteErrorKind> {
    match *op {
      Token::Lt => Ok(x < y),
      Token::Le => Ok(x <= y),
      Token::Gt => Ok(x > y),
      Token::Ge => Ok(x >= y),
      Token::Eql => Ok(x == y),
      Token::Ne => Ok(x != y),
      _ => Err(ExecuteErrorKind::BadCmpOp(op.clone())),
    }
  }

  fn ex_cmp_bool(op: &Token, x: bool, y: bool) -> Result<bool, ExecuteErrorKind> {
    match *op {
      Token::Eql => Ok(x == y),
      Token::Ne => Ok(x != y),
      Token::Lt | Token::Le | Token::Gt | Token::Ge => Err(ExecuteErrorKind::BadCmpOperands),
      _ => Err(ExecuteErrorKind::BadCmpOp(op.clone())),
    }
  }
}
