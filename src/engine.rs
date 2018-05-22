use code::Instr;
use codemap::CodeMap;
use core;
use data::Const;
use data::Data;
use data::Item;
use lexer::Token;
use module::Module;
use module::ModuleErrorKind;
use prelude;
use serde_yaml;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

struct RuntimeModule {
  pub scope: Item,
}

impl RuntimeModule {
  fn new(engine: &Engine) -> RuntimeModule {
    RuntimeModule {
      scope: Item {
        val: Data::new_table(),
        sup: Some(Box::new(engine.scope.clone())),
      },
    }
  }
}

// this is used as a bit of a control flow hack - `Return` and `Exception`
// aren't necessarily errors, but I'm using them with Rust's ? operator
// to skip a lot of boiler plate code later
#[derive(Debug)]
pub enum ExecuteErrorKind {
  AssertionFailure,
  BadArguments,
  BadOperator(Token),
  BadOperand,
  Break,
  Continue,
  EmptyStack,
  Exception,
  NotCallable,
  Return,
  Other,
}

#[derive(Debug)]
pub enum EngineErrorKind {
  ModuleError(ModuleErrorKind),
  ExecuteError(ExecuteErrorKind),
}

pub type Execute = Result<(), ExecuteErrorKind>;

pub struct Engine {
  pub map: CodeMap,
  mods: HashMap<String, Rc<Module>>,
  pub data_stack: Vec<Item>,
  scope: Item,
}

impl Engine {
  pub fn new() -> Engine {
    let mut ret = Engine {
      map: CodeMap::new(),
      mods: HashMap::new(),
      data_stack: Vec::new(),
      scope: Item {
        val: Data::new_table(),
        sup: None,
      },
    };

    prelude::insert_prelude(&mut ret.scope);

    ret
  }

  pub fn import(&mut self, filename: &str) -> Result<(()), EngineErrorKind> {
    let module = match Module::from_file(&mut self.map, filename) {
      Err(why) => return Err(EngineErrorKind::ModuleError(why)),
      Ok(module) => module,
    };

    let mut runtime = RuntimeModule::new(self);

    let rc_module = Rc::new(module);

    self.mods.insert(rc_module.name.clone(), rc_module.clone());

    // println!("{}: {}", filename, serde_yaml::to_string(rc_module.as_ref()).unwrap());

    match self.ex_many(&rc_module, &mut runtime, &rc_module.code) {
      Ok(_) => {}
      Err(ExecuteErrorKind::Return) => {}
      Err(why) => return Err(EngineErrorKind::ExecuteError(why)),
    }

    Ok(())
  }

  pub fn pop(&mut self) -> Result<Item, ExecuteErrorKind> {
    match self.data_stack.pop() {
      Some(x) => Ok(x),
      None => Err(ExecuteErrorKind::EmptyStack),
    }
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
    /*
    println!("EXECUTE {:?} in {:?}", instr, module.name);
    for item in self.data_stack.iter().rev() {
      println!("  {}", item.to_string());
    }
    */

    match *instr {
      Instr::PushConst(x) => {
        self.data_stack.push(module.consts[x].to_item());
      }

      Instr::PushScope => {
        self.data_stack.push(runtime.scope.clone());
      }

      Instr::PushFunc(x) => {
        self.data_stack.push(Item {
          val: Data::Func(x, module.name.clone()),
          sup: Some(Box::new(Item {
            val: Data::new_table(),
            sup: Some(Box::new(runtime.scope.clone())),
          })),
        });
      }

      Instr::Call => match self.data_stack.pop() {
        None => return Err(ExecuteErrorKind::EmptyStack),
        Some(func) => match func {
          Item {
            val: Data::Func(val, ref mname),
            sup: Some(ref sup),
          } => {
            let new_module = self.mods[mname].clone();
            let mut new_scope = Item {
              val: Data::new_table(),
              sup: Some(sup.clone()),
            };

            mem::swap(&mut new_scope, &mut runtime.scope);
            let func = new_module.funcs[val].clone();
            self.ex(&new_module, runtime, &func)?;
            mem::swap(&mut new_scope, &mut runtime.scope);
          }
          Item {
            val: Data::Func(val, ref mname),
            sup: None,
          } => {
            let new_module = self.mods[mname].clone();
            let func = module.funcs[val].clone();
            self.ex(&new_module, runtime, &func)?;
          }
          Item {
            val: Data::Rust(ref callable),
            sup: _,
          } => {
            callable.0(self)?;
          }
          _ => return Err(ExecuteErrorKind::NotCallable),
        },
      },

      Instr::Set => {
        core::set(self)?;
      }

      Instr::Get => {
        core::get(self)?;
      }

      Instr::MethodGet => {
        // MethodGet differs from regular Get by pushing the scope
        // back onto the stack. this is so method calls can
        // only evaluate the owner a single time.

        // this should guarantee that we can pop/unwrap twice
        if self.data_stack.len() < 2 {
          return Err(ExecuteErrorKind::EmptyStack);
        }

        let scope = self.data_stack.pop().unwrap();
        let key = self.data_stack.pop().unwrap();
        let val = scope.get_key(&key.val);
        self.data_stack.push(scope);
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

      Instr::BinOp(ref op) => match op {
        Token::Sup => core::bin::sup(self)?,
        Token::Cat => core::bin::cat(self)?,
        Token::Add => core::bin::add(self)?,
        Token::Sub => core::bin::sub(self)?,
        Token::Mul => core::bin::mul(self)?,
        Token::Div => core::bin::div(self)?,
        _ => return Err(ExecuteErrorKind::BadOperator(op.clone())),
      },

      Instr::UnOp(ref op) => match op {
        Token::Mul => core::un::mul(self)?,
        Token::Sub => core::un::sub(self)?,
        Token::Not => core::un::not(self)?,
        Token::Cat => core::un::cat(self)?,
        _ => return Err(ExecuteErrorKind::BadOperator(op.clone())),
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
        (Some(_), op) => return Err(ExecuteErrorKind::BadOperator(op.clone())),
        (None, _) => return Err(ExecuteErrorKind::EmptyStack),
      },

      Instr::CmpOp(ref op, chain) => {
        let rhs = self.pop()?;
        let lhs = self.pop()?;

        let result = match op {
          Token::Eql => core::cmp::eq_aux(&lhs, &rhs)?,
          Token::Ne => core::cmp::ne_aux(&lhs, &rhs)?,
          Token::Lt => core::cmp::lt_aux(&lhs, &rhs)?,
          Token::Gt => core::cmp::gt_aux(&lhs, &rhs)?,
          Token::Le => core::cmp::le_aux(&lhs, &rhs)?,
          Token::Ge => core::cmp::ge_aux(&lhs, &rhs)?,
          _ => return Err(ExecuteErrorKind::BadOperator(op.clone())),
        };

        match (chain, result) {
          (true, false) => {
            self.data_stack.push(Data::Bool(result).into_item());
            return Err(ExecuteErrorKind::Return);
          }
          (true, true) => {
            self.data_stack.push(rhs);
          }
          (false, _) => {
            self.data_stack.push(Data::Bool(result).into_item());
          }
        }
      }

      Instr::If(ref body) => match self.data_stack.pop() {
        Some(x) => {
          if x.truth() {
            self.ex_many(module, runtime, body)?;
          }
        }
        None => return Err(ExecuteErrorKind::EmptyStack),
      },

      Instr::IfElse(ref body, ref els) => match self.data_stack.pop() {
        Some(x) => match x.truth() {
          true => self.ex_many(module, runtime, body)?,
          false => self.ex_many(module, runtime, els)?,
        },
        None => return Err(ExecuteErrorKind::EmptyStack),
      },

      Instr::Loop(ref body) => loop {
        match self.ex_many(module, runtime, body) {
          Ok(_) => {}
          Err(ExecuteErrorKind::Break) => break,
          Err(ExecuteErrorKind::Continue) => continue,
          err => return err,
        }
      },

      Instr::Returnable(ref body) => match self.ex_many(module, runtime, body) {
        Ok(_) => {}
        Err(ExecuteErrorKind::Return) => {}
        err => return err,
      },

      Instr::Catch(ref body) => match self.ex_many(module, runtime, body) {
        Ok(_) => {}
        Err(ExecuteErrorKind::Exception) => {}
        err => return err,
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

      Instr::ForBreak => match self.data_stack.pop() {
        Some(x) => {
          if x.null() {
            return Err(ExecuteErrorKind::Break);
          }
          self.data_stack.push(x);
        }
        _ => {}
      },

      _ => {
        println!("WARNING: Unable to use instruction: {:?}", instr);
      }
    }

    Ok(())
  }
}

#[cfg(test)]
#[path = "./tests/engine.rs"]
mod tests;
