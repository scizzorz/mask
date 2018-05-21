int_mt = table()
int_mt.double = |x| x * 2

six = 6 :: int_mt
eval_count = 0
f = fn()
  ..eval_count = eval_count + 1
  return 5 :: int_mt

assert(six == 6)
assert(six.double == int_mt.double)
assert(six.double(six) == 12)
assert(six:double() == 12)
assert(f():double() == 10)
assert(eval_count == 1)
