use FloatBase;
use data::Const;
use data::Item;
use engine::Engine;
use engine::Execute;
use error::ExecuteControl;
use float;
use std::mem;

pub fn eq_aux(_: &mut Engine, lhs: &Item, rhs: &Item) -> Result<bool, ExecuteControl> {
  use data::Data::*;
  let res = match (&lhs.val, &rhs.val) {
    (&Null, &Null) => true,
    (&Int(x), &Int(y)) => x == y,
    (&Int(x), &Float(y)) => float::from(x as FloatBase) == y,
    (&Float(x), &Int(y)) => x == float::from(y as FloatBase),
    (&Float(x), &Float(y)) => x == y,
    (&Bool(x), &Bool(y)) => x == y,
    (&Str(ref x), &Str(ref y)) => x == y,
    (&Func { id: xid, module: ref xmod, nargs: xnargs }, &Func { id: yid, module: ref ymod, nargs: ynargs }) => (xid == yid) && (xmod == ymod) && (xnargs == ynargs),
    (&Rust(ref x), &Rust(ref y)) => {
      let xaddr = unsafe { mem::transmute::<_, u128>(x.0) };
      let yaddr = unsafe { mem::transmute::<_, u128>(y.0) };
      xaddr == yaddr
    }
    (&Table(ref x), &Table(ref y)) => {
      let xaddr = unsafe { mem::transmute::<_, u64>(x.clone()) };
      let yaddr = unsafe { mem::transmute::<_, u64>(y.clone()) };
      xaddr == yaddr
    }
    _ => false,
  };

  Ok(res)
}

pub fn ne_aux(_: &mut Engine, lhs: &Item, rhs: &Item) -> Result<bool, ExecuteControl> {
  use data::Data::*;
  let res = match (&lhs.val, &rhs.val) {
    (&Null, &Null) => false,
    (&Int(x), &Int(y)) => x != y,
    (&Int(x), &Float(y)) => float::from(x as FloatBase) != y,
    (&Float(x), &Int(y)) => x != float::from(y as FloatBase),
    (&Float(x), &Float(y)) => x != y,
    (&Bool(x), &Bool(y)) => x != y,
    (&Str(ref x), &Str(ref y)) => x != y,
    (&Func { id: xid, module: ref xmod, nargs: xnargs }, &Func { id: yid, module: ref ymod, nargs: ynargs }) => (xid != yid) || (xmod != ymod) || (xnargs != ynargs),
    (&Rust(ref x), &Rust(ref y)) => {
      let xaddr = unsafe { mem::transmute::<_, u128>(x.0) };
      let yaddr = unsafe { mem::transmute::<_, u128>(y.0) };
      xaddr != yaddr
    }
    (&Table(ref x), &Table(ref y)) => {
      let xaddr = unsafe { mem::transmute::<_, u64>(x.clone()) };
      let yaddr = unsafe { mem::transmute::<_, u64>(y.clone()) };
      xaddr != yaddr
    }
    _ => true,
  };

  Ok(res)
}

pub fn lt_aux(engine: &mut Engine, lhs: &Item, rhs: &Item) -> Result<bool, ExecuteControl> {
  use data::Data::*;
  let res = match (&lhs.val, &rhs.val) {
    (&Int(x), &Int(y)) => x < y,
    (&Int(x), &Float(y)) => float::from(x as FloatBase) < y,
    (&Float(x), &Int(y)) => x < float::from(y as FloatBase),
    (&Float(x), &Float(y)) => x < y,
    (&Bool(x), &Bool(y)) => x < y,
    (&Str(ref x), &Str(ref y)) => x < y,
    _ => {
      let exc = engine.bad_arguments.clone();
      engine.push(exc);
      return Err(ExecuteControl::Exception);
    }
  };

  Ok(res)
}

pub fn gt_aux(engine: &mut Engine, lhs: &Item, rhs: &Item) -> Result<bool, ExecuteControl> {
  use data::Data::*;
  let res = match (&lhs.val, &rhs.val) {
    (&Int(x), &Int(y)) => x > y,
    (&Int(x), &Float(y)) => float::from(x as FloatBase) > y,
    (&Float(x), &Int(y)) => x > float::from(y as FloatBase),
    (&Float(x), &Float(y)) => x > y,
    (&Bool(x), &Bool(y)) => x > y,
    (&Str(ref x), &Str(ref y)) => x > y,
    _ => {
      let exc = engine.bad_arguments.clone();
      engine.push(exc);
      return Err(ExecuteControl::Exception);
    }
  };

  Ok(res)
}

pub fn le_aux(engine: &mut Engine, lhs: &Item, rhs: &Item) -> Result<bool, ExecuteControl> {
  use data::Data::*;
  let res = match (&lhs.val, &rhs.val) {
    (&Int(x), &Int(y)) => x <= y,
    (&Int(x), &Float(y)) => float::from(x as FloatBase) <= y,
    (&Float(x), &Int(y)) => x <= float::from(y as FloatBase),
    (&Float(x), &Float(y)) => x <= y,
    (&Bool(x), &Bool(y)) => x <= y,
    (&Str(ref x), &Str(ref y)) => x <= y,
    _ => {
      let exc = engine.bad_arguments.clone();
      engine.push(exc);
      return Err(ExecuteControl::Exception);
    }
  };

  Ok(res)
}

pub fn ge_aux(engine: &mut Engine, lhs: &Item, rhs: &Item) -> Result<bool, ExecuteControl> {
  use data::Data::*;
  let res = match (&lhs.val, &rhs.val) {
    (&Int(x), &Int(y)) => x >= y,
    (&Int(x), &Float(y)) => float::from(x as FloatBase) >= y,
    (&Float(x), &Int(y)) => x >= float::from(y as FloatBase),
    (&Float(x), &Float(y)) => x >= y,
    (&Bool(x), &Bool(y)) => x >= y,
    (&Str(ref x), &Str(ref y)) => x >= y,
    _ => {
      let exc = engine.bad_arguments.clone();
      engine.push(exc);
      return Err(ExecuteControl::Exception);
    }
  };

  Ok(res)
}

pub fn eq(engine: &mut Engine) -> Execute {
  let rhs = engine.pop()?;
  let lhs = engine.pop()?;
  let res = eq_aux(engine, &lhs, &rhs)?;
  engine.data_stack.push(Const::Bool(res).to_item());
  Ok(())
}

pub fn ne(engine: &mut Engine) -> Execute {
  let rhs = engine.pop()?;
  let lhs = engine.pop()?;
  let res = ne_aux(engine, &lhs, &rhs)?;
  engine.data_stack.push(Const::Bool(res).to_item());
  Ok(())
}

pub fn lt(engine: &mut Engine) -> Execute {
  let rhs = engine.pop()?;
  let lhs = engine.pop()?;
  let res = lt_aux(engine, &lhs, &rhs)?;
  engine.data_stack.push(Const::Bool(res).to_item());
  Ok(())
}

pub fn gt(engine: &mut Engine) -> Execute {
  let rhs = engine.pop()?;
  let lhs = engine.pop()?;
  let res = gt_aux(engine, &lhs, &rhs)?;
  engine.data_stack.push(Const::Bool(res).to_item());
  Ok(())
}

pub fn le(engine: &mut Engine) -> Execute {
  let rhs = engine.pop()?;
  let lhs = engine.pop()?;
  let res = le_aux(engine, &lhs, &rhs)?;
  engine.data_stack.push(Const::Bool(res).to_item());
  Ok(())
}

pub fn ge(engine: &mut Engine) -> Execute {
  let rhs = engine.pop()?;
  let lhs = engine.pop()?;
  let res = ge_aux(engine, &lhs, &rhs)?;
  engine.data_stack.push(Const::Bool(res).to_item());
  Ok(())
}
