use std::fs;
use std::io::Write;

fn main() {
  // open the generated engine tests file
  let out_dir = std::env::var("OUT_DIR").unwrap();
  let destination = std::path::Path::new(&out_dir).join("engine_tests.rs");
  let mut output_file = std::fs::File::create(&destination).unwrap();

  // remove any cache files from the tests dir
  let paths = fs::read_dir("./tests").unwrap();
  for path in paths {
    let path = path.unwrap().path();
    let file_name = path.file_name().unwrap().to_str().unwrap();

    if file_name.ends_with(".msc") {
      fs::remove_file(&path).unwrap();
    }
  }

  // then generate a test for each sample
  let paths = fs::read_dir("./tests").unwrap();
  for path in paths {
    let path = path.unwrap().path();
    let file_name = path.to_str().unwrap();

    // there has to be a better way for this, no?
    let mut test_name = String::from(path.file_name().unwrap().to_str().unwrap());
    let new_len = test_name.len() - 3;
    test_name.truncate(new_len);

    // skip non .ms files
    if !file_name.ends_with(".ms") {
      continue;
    }

    write!(
      output_file,
      "#[test]
fn {test_name}() {{
  let mut engine = Engine::new();
  engine.import(\"{file_name}\").unwrap();
}}
",
      test_name = test_name,
      file_name = file_name,
    ).unwrap();
  }
}
