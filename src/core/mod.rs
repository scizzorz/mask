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

pub mod cmp;
pub mod bin;
