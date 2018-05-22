use FloatBase;
use data::Const;
use data::Data;
use data::Item;
use engine::Engine;
use engine::Execute;
use engine::ExecuteErrorKind;
use float;

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
  let val = engine.pop()?;

  use data::Data::*;
  let ret = match val.val {
    Int(x) => Int(-x),
    Float(x) => Float(float::from(-x.into_inner())),
    _ => return Err(ExecuteErrorKind::BadOperand),
  };

  engine.data_stack.push(ret.into_item());

  Ok(())
}

pub fn neg(engine: &mut Engine) -> Execute {
  Ok(())
}

pub fn not(engine: &mut Engine) -> Execute {
  let val = engine.pop()?;
  let ret = Const::Bool(!val.truth());
  engine.data_stack.push(ret.into_item());
  Ok(())
}

pub fn cat(engine: &mut Engine) -> Execute {
  let val = engine.pop()?;
  let ret = Const::Str(val.to_string());
  engine.data_stack.push(ret.into_item());
  Ok(())
}
