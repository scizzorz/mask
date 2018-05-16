use super::*;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;

#[test]
fn test_samples() {
  // ignoring all possible FS failures; they'll cause the test to fail
  // whether I handle it or whether unwrap panics

  // first remove any cache files
  let paths = fs::read_dir("./tests").unwrap();
  for path in paths {
    let path = path.unwrap().path();
    let file_name = path.file_name().unwrap().to_str().unwrap();

    // remove .msc files
    if file_name.ends_with(".msc") {
      fs::remove_file(&path);
    }
  }

  // then compile everything!
  let paths = fs::read_dir("./tests").unwrap();
  for path in paths {
    let path = path.unwrap().path();
    let file_name = path.to_str().unwrap();

    // skip non .ms files
    if !file_name.ends_with(".ms") {
      continue;
    }

    // compile / run
    let mut engine = Engine::new();
    match engine.import(file_name) {
      Ok(x) => {},
      Err(why) => panic!("Unable to import {:?}: {:?}", path, why),
    }
  }
}
