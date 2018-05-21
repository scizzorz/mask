f = fn()
  panic(5)

x = catch
  panic(true)
assert(x == true)

x = catch
  f()
assert(x == 5)
