use core;
use data::Const;
use data::Data;
use data::Item;
use data::RustFunc;
use gc::Gc;

fn insert_item(scope: &mut Item, key: &str, val: Item) {
  scope.set_key(Const::Str(String::from(key)).into_data(), val);
}

fn insert_data(scope: &mut Item, key: &str, val: Data) {
  insert_item(scope, key, val.into_item());
}

pub fn insert_prelude(scope: &mut Item) {
  insert_data(scope, "print", Data::Rust(RustFunc(&core::print)));
  insert_data(scope, "panic", Data::Rust(RustFunc(&core::panic)));
  insert_data(scope, "assert", Data::Rust(RustFunc(&core::assert)));
  insert_data(scope, "import", Data::Rust(RustFunc(&core::import)));
  insert_data(scope, "table", Data::Rust(RustFunc(&core::table)));
  insert_data(scope, "get", Data::Rust(RustFunc(&core::get)));
  insert_data(scope, "set", Data::Rust(RustFunc(&core::set_mask)));
}
