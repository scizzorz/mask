use data::Const;
use data::Data;
use data::Item;
use data::RustFunc;
use engine::Engine;
use engine::Execute;
use engine::ExecuteErrorKind;

pub fn print_func(engine: &mut Engine) -> Execute {
  match engine.data_stack.pop() {
    Some(x) => println!("{}", x.to_string()),
    None => return Err(ExecuteErrorKind::EmptyStack),
  }

  engine.data_stack.push(Const::Null.into_item());

  Ok(())
}

pub fn panic_func(_: &mut Engine) -> Execute {
  return Err(ExecuteErrorKind::Exception);
}

fn insert_item(scope: &mut Item, key: &str, val: Item) {
  scope.set_key(Const::Str(String::from(key)).into_data(), val);
}

fn insert_data(scope: &mut Item, key: &str, val: Data) {
  insert_item(scope, key, val.into_item());
}

pub fn insert_prelude(scope: &mut Item) {
  insert_data(scope, "print", Data::Rust(RustFunc(&print_func)));
  insert_data(scope, "panic", Data::Rust(RustFunc(&panic_func)));
}
