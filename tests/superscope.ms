f = fn(n)
  assert(n == 30)
  assert(.n == 20)
  assert(..n == 10)

  ..n = 40
  .n = 50


n = 10
f.n = 20
f(30)

assert(f.n == 50)
assert(n == 40)
