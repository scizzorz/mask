range = fn(max)
  cur = 0

  f = fn()
    tmp = cur
    if cur < max
      .cur = cur + 1
      return tmp

  return f

r = range(5)

sum = 0
for x in r
  sum = sum + x

assert(sum == 10)

r.cur = 10
r.max = 15

for x in r
  sum = sum + x

assert(sum == 70)
