use data::Const;
use data::Data;
use data::Item;
use engine::Engine;
use engine::EngineErrorKind;
use engine::Execute;
use engine::ExecuteErrorKind;
use float;
use float_base;
use std::mem;

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

pub fn cmp_eq(engine: &mut Engine) -> Execute {
  let rhs = engine.pop()?;
  let lhs = engine.pop()?;

  use data::Data::*;
  let res = match (&lhs.val, &rhs.val) {
    (&Null, &Null) => true,
    (&Int(x), &Int(y)) => x == y,
    (&Int(x), &Float(y)) => float::from(x as float_base) == y,
    (&Float(x), &Int(y)) => x == float::from(y as float_base),
    (&Float(x), &Float(y)) => x == y,
    (&Bool(x), &Bool(y)) => x == y,
    (&Str(ref x), &Str(ref y)) => x == y,
    (&Func(xi, ref xm), &Func(yi, ref ym)) => (xi == yi) && (xm == ym),
    (&Rust(ref x), &Rust(ref y)) => {
      let xaddr = unsafe { mem::transmute::<_, u128>(x.0)  };
      let yaddr = unsafe { mem::transmute::<_, u128>(y.0)  };
      xaddr == yaddr
    }
    (&Table(ref x), &Table(ref y)) => {
      let xaddr = unsafe { mem::transmute::<_, u64>(x.clone())  };
      let yaddr = unsafe { mem::transmute::<_, u64>(y.clone())  };
      xaddr == yaddr
    }
    _ => false,
  };

  Ok(())
}
