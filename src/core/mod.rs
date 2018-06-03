use data::Const;
use data::Data;
use data::Item;
use engine::Engine;
use engine::EngineErrorKind;
use engine::Execute;
use engine::ExecuteErrorKind;

pub fn table(engine: &mut Engine) -> Execute {
  engine.push(Item {
    val: Data::new_table(),
    sup: None,
  });

  Ok(())
}

pub fn print(engine: &mut Engine) -> Execute {
  let x = engine.pop()?;
  println!("{}", x.to_string());
  engine.push(Const::Null.into_item());
  Ok(())
}

pub fn import(engine: &mut Engine) -> Execute {
  let top = engine.pop()?;
  match top {
    Item {
      val: Data::Str(ref x),
      sup: _,
    } => match engine.import(x) {
      Err(EngineErrorKind::ModuleError(_)) => return Err(ExecuteErrorKind::Other),
      Err(EngineErrorKind::ExecuteError(x)) => return Err(x),
      Ok(_) => {}
    },
    _ => {
      let exc = engine.bad_arguments.clone();
      engine.panic(exc)?;
    }
  }

  Ok(())
}

pub fn assert(engine: &mut Engine) -> Execute {
  let x = engine.pop()?;
  if !x.truth() {
    let exc = engine.assertion_failure.clone();
    engine.panic(exc)?;
  }

  engine.push(Const::Null.into_item());

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

  engine.push(val);

  Ok(())
}

pub fn set_mask(engine: &mut Engine) -> Execute {
  let mut scope = engine.pop()?;
  let key = engine.pop()?;
  let val = engine.pop()?;

  scope.set_key(key.val.clone(), val.clone());

  engine.push(val);

  Ok(())
}

pub mod bin;
pub mod cmp;
pub mod un;
