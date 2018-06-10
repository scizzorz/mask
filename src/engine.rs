use code::Instr;
use codemap::CodeMap;
use core;
use data::Const;
use data::Data;
use data::Item;
use error::ExecuteControl;
use lexer::Token;
use module::Module;
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

pub type Execute = Result<(), ExecuteControl>;

pub struct Engine {
  pub map: CodeMap,
  mods: HashMap<String, Rc<Module>>,
  pub data_stack: Vec<Item>,
  scope: Item,
  pub assertion_failure: Item,
  pub bad_arguments: Item,
  pub bad_operator: Item,
  pub empty_stack: Item,
  pub not_callable: Item,
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
      assertion_failure: Const::Str(String::from("Assetion failure")).into_item(),
      bad_arguments: Const::Str(String::from("Bad arguments")).into_item(),
      bad_operator: Const::Str(String::from("Bad operator")).into_item(),
      empty_stack: Const::Str(String::from("Empty stack")).into_item(),
      not_callable: Const::Str(String::from("Not callable")).into_item(),
    };

    prelude::insert_prelude(&mut ret.scope);

    ret
  }

  pub fn import(&mut self, filename: &str) -> Execute {
    let module = match Module::from_file(&mut self.map, filename) {
      Err(why) => {
        let exc = Const::Str(format!("{:?}", why)).into_item();
        return self.panic(exc);
      }
      Ok(module) => module,
    };

    let mut runtime = RuntimeModule::new(self);

    let rc_module = Rc::new(module);

    self.mods.insert(rc_module.name.clone(), rc_module.clone());

    // println!("{}: {}", filename, serde_yaml::to_string(rc_module.as_ref()).unwrap());

    match self.ex_many(&rc_module, &mut runtime, &rc_module.code) {
      Ok(_) => {}
      Err(ExecuteControl::Return) => {}
      Err(why) => return Err(why),
    }

    Ok(())
  }

  pub fn pop(&mut self) -> Result<Item, ExecuteControl> {
    match self.data_stack.pop() {
      Some(x) => Ok(x),
      None => {
        let exc = self.empty_stack.clone();
        self.panic(exc)?;
        unreachable!();
      }
    }
  }

  pub fn push(&mut self, item: Item) {
    self.data_stack.push(item);
  }

  pub fn panic(&mut self, error: Item) -> Execute {
    self.push(error);
    return Err(ExecuteControl::Exception);
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

      Instr::Call(nargs) => {
        let func = self.pop()?;
        match func {
          // function with scope
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

          // function with no scope
          Item {
            val: Data::Func(val, ref mname),
            sup: None,
          } => {
            let new_module = self.mods[mname].clone();
            let func = module.funcs[val].clone();
            self.ex(&new_module, runtime, &func)?;
          }

          // Rust function; scope is ignored
          // FIXME maybe rust functions should get their scope swapped too...?
          Item {
            val: Data::Rust(ref callable),
            sup: _,
          } => {
            callable.0(self)?;
          }

          // not callable!
          _ => {
            let exc = self.not_callable.clone();
            self.panic(exc)?;
          }
        }
      }

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

        let scope = self.pop()?;
        let key = self.pop()?;
        let val = scope.get_key(&key.val);
        self.data_stack.push(scope);
        self.data_stack.push(val);
      }

      Instr::Pop => {
        self.pop()?;
      }

      Instr::Dup => {
        let x = self.pop()?;
        self.data_stack.push(x.clone());
        self.data_stack.push(x);
      }

      Instr::Nop => {}

      Instr::BinOp(ref op) => match op {
        Token::Sup => core::bin::sup(self)?,
        Token::Cat => core::bin::cat(self)?,
        Token::Add => core::bin::add(self)?,
        Token::Sub => core::bin::sub(self)?,
        Token::Mul => core::bin::mul(self)?,
        Token::Div => core::bin::div(self)?,
        _ => {
          let exc = self.bad_operator.clone();
          self.panic(exc)?;
        }
      },

      Instr::UnOp(ref op) => match op {
        Token::Mul => core::un::mul(self)?,
        Token::Sub => core::un::sub(self)?,
        Token::Not => core::un::not(self)?,
        Token::Cat => core::un::cat(self)?,
        _ => {
          let exc = self.bad_operator.clone();
          self.panic(exc)?;
        }
      },

      Instr::LogicOp(ref op) => match (self.data_stack.pop(), op) {
        (Some(val), &Token::And) => {
          if !val.truth() {
            self.data_stack.push(val);
            return Err(ExecuteControl::Return);
          }
        }
        (Some(val), &Token::Or) => {
          if val.truth() {
            self.data_stack.push(val);
            return Err(ExecuteControl::Return);
          }
        }
        (Some(_), _) => {
          let exc = self.bad_operator.clone();
          self.panic(exc)?;
        }
        (None, _) => {
          let exc = self.empty_stack.clone();
          self.panic(exc)?;
        }
      },

      Instr::CmpOp(ref op, chain) => {
        let rhs = self.pop()?;
        let lhs = self.pop()?;

        let result = match op {
          Token::Eql => core::cmp::eq_aux(self, &lhs, &rhs)?,
          Token::Ne => core::cmp::ne_aux(self, &lhs, &rhs)?,
          Token::Lt => core::cmp::lt_aux(self, &lhs, &rhs)?,
          Token::Gt => core::cmp::gt_aux(self, &lhs, &rhs)?,
          Token::Le => core::cmp::le_aux(self, &lhs, &rhs)?,
          Token::Ge => core::cmp::ge_aux(self, &lhs, &rhs)?,
          _ => {
            let exc = self.bad_operator.clone();
            return self.panic(exc);
          }
        };

        match (chain, result) {
          (true, false) => {
            self.data_stack.push(Data::Bool(result).into_item());
            return Err(ExecuteControl::Return);
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
        None => {
          let exc = self.empty_stack.clone();
          self.panic(exc)?;
        }
      },

      Instr::IfElse(ref body, ref els) => match self.data_stack.pop() {
        Some(x) => match x.truth() {
          true => self.ex_many(module, runtime, body)?,
          false => self.ex_many(module, runtime, els)?,
        },
        None => {
          let exc = self.empty_stack.clone();
          self.panic(exc)?;
        }
      },

      Instr::Loop(ref body) => loop {
        match self.ex_many(module, runtime, body) {
          Ok(_) => {}
          Err(ExecuteControl::Break) => break,
          Err(ExecuteControl::Continue) => continue,
          err => return err,
        }
      },

      Instr::Returnable(ref body) => match self.ex_many(module, runtime, body) {
        Ok(_) => {}
        Err(ExecuteControl::Return) => {}
        err => return err,
      },

      Instr::Catch(ref body) => {
        let enter_stack = self.data_stack.len();
        match self.ex_many(module, runtime, body) {
          Ok(_) => {}
          Err(ExecuteControl::Exception) => {
            let exc = self.pop()?;
            self.data_stack.truncate(enter_stack);
            self.data_stack.push(exc);
          }
          err => return err,
        }
      }

      Instr::Return => {
        return Err(ExecuteControl::Return);
      }

      Instr::Break => {
        return Err(ExecuteControl::Break);
      }

      Instr::Continue => {
        return Err(ExecuteControl::Continue);
      }

      Instr::ForBreak => match self.data_stack.pop() {
        Some(x) => {
          if x.null() {
            return Err(ExecuteControl::Break);
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
