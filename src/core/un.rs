use data::Item;
use data::Data;
use data::Const;
use engine::Engine;
use engine::Execute;
use engine::ExecuteErrorKind;
use float;
use FloatBase;

pub fn star(engine: &mut Engine) -> Execute {
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
