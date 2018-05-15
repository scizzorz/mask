f = fn(n)
  print n == n
  print .n == (*local).n
  print ..n == (**local).n


n = 10
f.n = 20
f(30)
