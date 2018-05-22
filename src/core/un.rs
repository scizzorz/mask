use data::Item;
use data::Data;
use data::Const;
use engine::Engine;
use engine::Execute;
use engine::ExecuteErrorKind;
use float;
use FloatBase;

pub fn mul(engine: &mut Engine) -> Execute {
  let val = engine.pop()?;

  match val.sup {
    Some(ref sup) => {
      engine.data_stack.push(*sup.clone());
    }
    None => {
      engine.data_stack.push(Const::Null.into_item());
    }
  }

  Ok(())
}

pub fn sub(engine: &mut Engine) -> Execute {
  Ok(())
}

pub fn neg(engine: &mut Engine) -> Execute {
  Ok(())
}

pub fn not(engine: &mut Engine) -> Execute {
  Ok(())
}

pub fn dol(engine: &mut Engine) -> Execute {
  Ok(())
}
