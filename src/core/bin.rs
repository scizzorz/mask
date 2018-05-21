use data::Item;
use data::Data;
use data::Const;
use engine::Engine;
use engine::Execute;
use engine::ExecuteErrorKind;
use float;
use float_base;

pub fn sup(engine: &mut Engine) -> Execute {
  let rhs = engine.pop()?;
  let lhs = engine.pop()?;

  let ret = Item {
    sup: match rhs.val {
      Data::Null => None,
      _ => Some(Box::new(rhs)),
    },
    val: lhs.val.clone(),
  };

  engine.data_stack.push(ret);

  Ok(())
}

pub fn cat(engine: &mut Engine) -> Execute {
  let rhs = engine.pop()?;
  let lhs = engine.pop()?;

  let mut ret = lhs.to_string();
  ret.push_str(&rhs.to_string());
  let ret = Const::Str(ret).into_item();

  engine.data_stack.push(ret);

  Ok(())
}

pub fn add(engine: &mut Engine) -> Execute {
  let rhs = engine.pop()?;
  let lhs = engine.pop()?;

  use data::Data::*;
  let ret = match (&lhs.val, &rhs.val) {
    (&Int(x), &Int(y)) => Int(x + y),
    (&Int(x), &Float(y)) => Float(float::from(x as float_base + y.into_inner())),
    (&Float(x), &Int(y)) => Float(float::from(x.into_inner() + y as float_base)),
    (&Float(x), &Float(y)) => Float(float::from(x.into_inner() + y.into_inner())),
    _ => return Err(ExecuteErrorKind::BadOperand),
  };

  engine.data_stack.push(ret.into_item());

  Ok(())
}

pub fn sub(engine: &mut Engine) -> Execute {
  let rhs = engine.pop()?;
  let lhs = engine.pop()?;

  use data::Data::*;
  let ret = match (&lhs.val, &rhs.val) {
    (&Int(x), &Int(y)) => Int(x - y),
    (&Int(x), &Float(y)) => Float(float::from(x as float_base - y.into_inner())),
    (&Float(x), &Int(y)) => Float(float::from(x.into_inner() - y as float_base)),
    (&Float(x), &Float(y)) => Float(float::from(x.into_inner() - y.into_inner())),
    _ => return Err(ExecuteErrorKind::BadOperand),
  };

  engine.data_stack.push(ret.into_item());

  Ok(())
}

pub fn mul(engine: &mut Engine) -> Execute {
  let rhs = engine.pop()?;
  let lhs = engine.pop()?;

  use data::Data::*;
  let ret = match (&lhs.val, &rhs.val) {
    (&Int(x), &Int(y)) => Int(x * y),
    (&Int(x), &Float(y)) => Float(float::from(x as float_base * y.into_inner())),
    (&Float(x), &Int(y)) => Float(float::from(x.into_inner() * y as float_base)),
    (&Float(x), &Float(y)) => Float(float::from(x.into_inner() * y.into_inner())),
    _ => return Err(ExecuteErrorKind::BadOperand),
  };

  engine.data_stack.push(ret.into_item());

  Ok(())
}

pub fn div(engine: &mut Engine) -> Execute {
  let rhs = engine.pop()?;
  let lhs = engine.pop()?;

  use data::Data::*;
  let ret = match (&lhs.val, &rhs.val) {
    (&Int(x), &Int(y)) => Float(float::from(x as float_base / y as float_base)),
    (&Int(x), &Float(y)) => Float(float::from(x as float_base / y.into_inner())),
    (&Float(x), &Int(y)) => Float(float::from(x.into_inner() / y as float_base)),
    (&Float(x), &Float(y)) => Float(float::from(x.into_inner() / y.into_inner())),
    _ => return Err(ExecuteErrorKind::BadOperand),
  };

  engine.data_stack.push(ret.into_item());

  Ok(())
}
