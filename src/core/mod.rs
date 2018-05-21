use data::Const;
use data::Data;
use data::Item;
use engine::Engine;
use engine::EngineErrorKind;
use engine::Execute;
use engine::ExecuteErrorKind;

pub fn table(engine: &mut Engine) -> Execute {
  engine.data_stack.push(Item {
    val: Data::new_table(),
    sup: None,
  });

  Ok(())
}

pub fn print(engine: &mut Engine) -> Execute {
  match engine.data_stack.pop() {
    Some(x) => println!("{}", x.to_string()),
    None => return Err(ExecuteErrorKind::EmptyStack),
  }

  engine.data_stack.push(Const::Null.into_item());

  Ok(())
}

pub fn import(engine: &mut Engine) -> Execute {
  match engine.data_stack.pop() {
    Some(Item {
      val: Data::Str(ref x),
      sup: _,
    }) => match engine.import(x) {
      Err(EngineErrorKind::ModuleError(_)) => return Err(ExecuteErrorKind::Other),
      Err(EngineErrorKind::ExecuteError(x)) => return Err(x),
      Ok(_) => {}
    },
    Some(_) => return Err(ExecuteErrorKind::BadArguments),
    None => return Err(ExecuteErrorKind::EmptyStack),
  }

  Ok(())
}

pub fn assert(engine: &mut Engine) -> Execute {
  match engine.data_stack.pop() {
    Some(x) => {
      if !x.truth() {
        return Err(ExecuteErrorKind::AssertionFailure);
      }
    }
    None => return Err(ExecuteErrorKind::EmptyStack),
  }

  engine.data_stack.push(Const::Null.into_item());

  Ok(())
}

pub fn panic(_: &mut Engine) -> Execute {
  return Err(ExecuteErrorKind::Exception);
}

pub fn set(engine: &mut Engine) -> Execute {
  let mut scope = engine.pop()?;
  let key = engine.pop()?;
  let val = engine.pop()?;

  scope.set_key(key.val.clone(), val);

  Ok(())
}

pub fn get(engine: &mut Engine) -> Execute {
  let scope = engine.pop()?;
  let key = engine.pop()?;
  let val = scope.get_key(&key.val);

  engine.data_stack.push(val);

  Ok(())
}

pub fn set_mask(engine: &mut Engine) -> Execute {
  let val = engine.pop()?;
  let key = engine.pop()?;
  let mut scope = engine.pop()?;

  scope.set_key(key.val.clone(), val.clone());

  engine.data_stack.push(val);

  Ok(())
}

pub mod cmp;
pub mod bin;
