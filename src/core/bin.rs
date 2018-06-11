use data::Const;
use data::Data;
use data::Item;
use engine::Engine;
use engine::Execute;
use float;
use FloatBase;

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
    (&Int(x), &Float(y)) => Float(float::from(x as FloatBase + y.into_inner())),
    (&Float(x), &Int(y)) => Float(float::from(x.into_inner() + y as FloatBase)),
    (&Float(x), &Float(y)) => Float(float::from(x.into_inner() + y.into_inner())),
    _ => {
      let exc = engine.bad_arguments.clone();
      return engine.panic(exc);
    }
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
    (&Int(x), &Float(y)) => Float(float::from(x as FloatBase - y.into_inner())),
    (&Float(x), &Int(y)) => Float(float::from(x.into_inner() - y as FloatBase)),
    (&Float(x), &Float(y)) => Float(float::from(x.into_inner() - y.into_inner())),
    _ => {
      let exc = engine.bad_arguments.clone();
      return engine.panic(exc);
    }
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
    (&Int(x), &Float(y)) => Float(float::from(x as FloatBase * y.into_inner())),
    (&Float(x), &Int(y)) => Float(float::from(x.into_inner() * y as FloatBase)),
    (&Float(x), &Float(y)) => Float(float::from(x.into_inner() * y.into_inner())),
    _ => {
      let exc = engine.bad_arguments.clone();
      return engine.panic(exc);
    }
  };

  engine.data_stack.push(ret.into_item());

  Ok(())
}

pub fn div(engine: &mut Engine) -> Execute {
  let rhs = engine.pop()?;
  let lhs = engine.pop()?;

  use data::Data::*;
  let ret = match (&lhs.val, &rhs.val) {
    (&Int(x), &Int(y)) => Float(float::from(x as FloatBase / y as FloatBase)),
    (&Int(x), &Float(y)) => Float(float::from(x as FloatBase / y.into_inner())),
    (&Float(x), &Int(y)) => Float(float::from(x.into_inner() / y as FloatBase)),
    (&Float(x), &Float(y)) => Float(float::from(x.into_inner() / y.into_inner())),
    _ => {
      let exc = engine.bad_arguments.clone();
      return engine.panic(exc);
    }
  };

  engine.data_stack.push(ret.into_item());

  Ok(())
}
