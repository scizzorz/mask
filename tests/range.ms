range = fn(max)
  cur = 0
  f = fn()
    tmp = cur
    if cur >= max
      return
    f.cur = cur + 1
    return tmp
  return f

r = range(5)

for x in r
  print x

r.cur = 10
r.max = 15

for x in r
  print x
