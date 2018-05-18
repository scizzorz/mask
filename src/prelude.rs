use engine::Engine;
use engine::Execute;
use engine::ExecuteErrorKind;
use data::Const;
use data::Data;
use data::Item;
use data::RustFunc;

pub fn print_func(engine: &mut Engine) -> Execute {
  match engine.data_stack.pop() {
    Some(x) => println!("{}", x.to_string()),
    None => return Err(ExecuteErrorKind::EmptyStack),
  }

  engine.data_stack.push(Const::Null.into_item());

  Ok(())
}

pub fn insert_prelude(scope: &mut Item) {
  let print_key = Const::Str(String::from("print")).into_data();
  let print_func = Data::Rust(RustFunc(&print_func)).into_item();

  scope.set_key(print_key, print_func);
}
