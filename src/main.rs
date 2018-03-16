extern crate rain;
extern crate clap;

use clap::App;
use clap::Arg;

fn main() {
  let argv = App::new("rain")
    .version("0.0.1")
    .author("John Weachock <jweachock@gmail.com>")
    .about("A programming language.")
    .arg(Arg::with_name("path")
         .takes_value(true)
         .index(1)
         .help("Rain module to execute"))
    .get_matches();

  let path = argv.value_of("path").unwrap_or(".");

  let tokens = rain::lexer::lex(path);
  println!("{:?}", tokens);
}
