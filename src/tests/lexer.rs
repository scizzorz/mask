use super::*;
use codemap::CodeMap;
use codemap::Spanned;

fn get_tokens(source: &str) -> Vec<Spanned<Token>> {
  let mut map = CodeMap::new();
  let file = map.add_file(String::from("_test"), String::from(source));
  lex(&file)
}

#[test]
fn lex_numbers() {
  let source = "0 5 05 5.3 1.234 1. 0. 0.0";
  let tokens = get_tokens(source);

  assert_eq!(tokens.len(), 10);
  assert_eq!(tokens[0].node, Int(0));
  assert_eq!(tokens[1].node, Int(5));
  assert_eq!(tokens[2].node, Int(5));
  assert_eq!(tokens[3].node, Float(float::from(5.3)));
  assert_eq!(tokens[4].node, Float(float::from(1.234)));
  assert_eq!(tokens[5].node, Float(float::from(1.0)));
  assert_eq!(tokens[6].node, Float(float::from(0.0)));
  assert_eq!(tokens[7].node, Float(float::from(0.0)));
  assert_eq!(tokens[8].node, End);
  assert_eq!(tokens[9].node, EOF);
}

#[test]
fn lex_string() {
  let source = "'hello' 'this\\nis\\nmultiline' 'this\\\\is\\\\escaped' 'this\\tis\\ttabbed' 'this
is
real
multiline' 'and this is unclosed";
  let tokens = get_tokens(source);

  assert_eq!(tokens.len(), 8);
  assert_eq!(tokens[0].node, Str(String::from("hello")));
  assert_eq!(tokens[1].node, Str(String::from("this\nis\nmultiline")));
  assert_eq!(tokens[2].node, Str(String::from("this\\is\\escaped")));
  assert_eq!(tokens[3].node, Str(String::from("this\tis\ttabbed")));
  assert_eq!(
    tokens[4].node,
    Str(String::from("this\nis\nreal\nmultiline"))
  );
  assert_eq!(
    tokens[5].node,
    UnclosedStr(String::from("and this is unclosed"))
  );
  assert_eq!(tokens[6].node, End);
  assert_eq!(tokens[7].node, EOF);
}

#[test]
fn lex_keywords() {
  let source = "and break catch continue else for fn if in loop or pass return while name true false null";
  let tokens = get_tokens(source);
  assert_eq!(tokens.len(), 20);
  assert_eq!(tokens[0].node, And);
  assert_eq!(tokens[1].node, Break);
  assert_eq!(tokens[2].node, Catch);
  assert_eq!(tokens[3].node, Continue);
  assert_eq!(tokens[4].node, Else);
  assert_eq!(tokens[5].node, For);
  assert_eq!(tokens[6].node, Func);
  assert_eq!(tokens[7].node, If);
  assert_eq!(tokens[8].node, In);
  assert_eq!(tokens[9].node, Loop);
  assert_eq!(tokens[10].node, Or);
  assert_eq!(tokens[11].node, Pass);
  assert_eq!(tokens[12].node, Return);
  assert_eq!(tokens[13].node, While);
  assert_eq!(tokens[14].node, Name(String::from("name")));
  assert_eq!(tokens[15].node, Bool(true));
  assert_eq!(tokens[16].node, Bool(false));
  assert_eq!(tokens[17].node, Null);
  assert_eq!(tokens[18].node, End);
  assert_eq!(tokens[19].node, EOF);
}

#[test]
fn lex_symbols() {
  let source = "-> = : : , . :: ; {} () [] +&@^/$*~!|%- == >= - > = <= < = != ! =";
  let tokens = get_tokens(source);
  assert_eq!(tokens.len(), 39);
  assert_eq!(tokens[0].node, Arr);
  assert_eq!(tokens[1].node, Ass);
  assert_eq!(tokens[2].node, Col);
  assert_eq!(tokens[3].node, Col);
  assert_eq!(tokens[4].node, Com);
  assert_eq!(tokens[5].node, Dot);
  assert_eq!(tokens[6].node, Sup);
  assert_eq!(tokens[7].node, Sem);

  assert_eq!(tokens[8].node, Cul);
  assert_eq!(tokens[9].node, Cur);
  assert_eq!(tokens[10].node, Pal);
  assert_eq!(tokens[11].node, Par);
  assert_eq!(tokens[12].node, Sql);
  assert_eq!(tokens[13].node, Sqr);

  assert_eq!(tokens[14].node, Add);
  assert_eq!(tokens[15].node, Amp);
  assert_eq!(tokens[16].node, At);
  assert_eq!(tokens[17].node, Car);
  assert_eq!(tokens[18].node, Div);
  assert_eq!(tokens[19].node, Dol);
  assert_eq!(tokens[20].node, Mul);
  assert_eq!(tokens[21].node, Neg);
  assert_eq!(tokens[22].node, Not);
  assert_eq!(tokens[23].node, Pipe);
  assert_eq!(tokens[24].node, Pct);
  assert_eq!(tokens[25].node, Sub);
  assert_eq!(tokens[26].node, Eql);
  assert_eq!(tokens[27].node, Ge);
  assert_eq!(tokens[28].node, Sub);
  assert_eq!(tokens[29].node, Gt);
  assert_eq!(tokens[30].node, Ass);
  assert_eq!(tokens[31].node, Le);
  assert_eq!(tokens[32].node, Lt);
  assert_eq!(tokens[33].node, Ass);
  assert_eq!(tokens[34].node, Ne);
  assert_eq!(tokens[35].node, Not);
  assert_eq!(tokens[36].node, Ass);
  assert_eq!(tokens[37].node, End);
  assert_eq!(tokens[38].node, EOF);
}

#[test]
fn lex_structure() {
  // this test uses a single trailing hash to avoid trailing whitespace errors in git and editors
  // the test uses trailing whitespace to test lexing them
  // (maybe Mask shouldn't allow trailing whitespace? who knows)
  let source = "
  #
 #
pass
if true # comment
  pass
else
  pass
   #
if true   #
  if doublenest
    pass
  normalexit
else
  if doublenest
    if triplenest
      pass
pass

# blank line with comment

# indented second comment

if true # should always evaluate
  pass

else
  # blank line with comment
  #
    pass

  #
  "
    .replace("#\n", "\n");
  let tokens = get_tokens(source.as_str());
  assert_eq!(tokens.len(), 64);
  assert_eq!(tokens[0].node, Pass);
  assert_eq!(tokens[1].node, End);
  assert_eq!(tokens[2].node, If);
  assert_eq!(tokens[3].node, Bool(true));
  assert_eq!(tokens[4].node, Enter);
  assert_eq!(tokens[5].node, Pass);
  assert_eq!(tokens[6].node, End);
  assert_eq!(tokens[7].node, Exit);
  assert_eq!(tokens[8].node, End);
  assert_eq!(tokens[9].node, Else);
  assert_eq!(tokens[10].node, Enter);
  assert_eq!(tokens[11].node, Pass);
  assert_eq!(tokens[12].node, End);
  assert_eq!(tokens[13].node, Exit);
  assert_eq!(tokens[14].node, End);
  assert_eq!(tokens[15].node, If);
  assert_eq!(tokens[16].node, Bool(true));
  assert_eq!(tokens[17].node, Enter);
  assert_eq!(tokens[18].node, If);
  assert_eq!(tokens[19].node, Name(String::from("doublenest")));
  assert_eq!(tokens[20].node, Enter);
  assert_eq!(tokens[21].node, Pass);
  assert_eq!(tokens[22].node, End);
  assert_eq!(tokens[23].node, Exit);
  assert_eq!(tokens[24].node, End);
  assert_eq!(tokens[25].node, Name(String::from("normalexit")));
  assert_eq!(tokens[26].node, End);
  assert_eq!(tokens[27].node, Exit);
  assert_eq!(tokens[28].node, End);
  assert_eq!(tokens[29].node, Else);
  assert_eq!(tokens[30].node, Enter);
  assert_eq!(tokens[31].node, If);
  assert_eq!(tokens[32].node, Name(String::from("doublenest")));
  assert_eq!(tokens[33].node, Enter);
  assert_eq!(tokens[34].node, If);
  assert_eq!(tokens[35].node, Name(String::from("triplenest")));
  assert_eq!(tokens[36].node, Enter);
  assert_eq!(tokens[37].node, Pass);
  assert_eq!(tokens[38].node, End);
  assert_eq!(tokens[39].node, Exit);
  assert_eq!(tokens[40].node, End);
  assert_eq!(tokens[41].node, Exit);
  assert_eq!(tokens[42].node, End);
  assert_eq!(tokens[43].node, Exit);
  assert_eq!(tokens[44].node, End);
  assert_eq!(tokens[45].node, Pass);
  assert_eq!(tokens[46].node, End);
  assert_eq!(tokens[47].node, If);
  assert_eq!(tokens[48].node, Bool(true));
  assert_eq!(tokens[49].node, Enter);
  assert_eq!(tokens[50].node, Pass);
  assert_eq!(tokens[51].node, End);
  assert_eq!(tokens[52].node, Exit);
  assert_eq!(tokens[53].node, End);
  assert_eq!(tokens[54].node, Else);
  assert_eq!(tokens[55].node, Enter);
  assert_eq!(tokens[56].node, Enter);
  assert_eq!(tokens[57].node, Pass);
  assert_eq!(tokens[58].node, End);
  assert_eq!(tokens[59].node, Exit);
  assert_eq!(tokens[60].node, End);
  assert_eq!(tokens[61].node, Exit);
  assert_eq!(tokens[62].node, End);
  assert_eq!(tokens[63].node, EOF);
}

#[test]
fn lex_trailing_exits() {
  let source = "if true
  pass";
  let tokens = get_tokens(source);
  assert_eq!(tokens.len(), 8);
  assert_eq!(tokens[0].node, If);
  assert_eq!(tokens[1].node, Bool(true));
  assert_eq!(tokens[2].node, Enter);
  assert_eq!(tokens[3].node, Pass);
  assert_eq!(tokens[4].node, End);
  assert_eq!(tokens[5].node, Exit);
  assert_eq!(tokens[6].node, End);
  assert_eq!(tokens[7].node, EOF);
}
