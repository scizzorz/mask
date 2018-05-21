fact = fn(n)
  if n == 1
    return 1
  return n * fact(n - 1)

fib = fn(n)
  if n == 0 or n == 1
    return 1
  return fib(n - 1) + fib(n - 2)

assert(fact(1) == 1)
assert(fact(2) == 2)
assert(fact(3) == 6)
assert(fact(4) == 24)
assert(fact(5) == 120)
assert(fib(0) == 1)
assert(fib(1) == 1)
assert(fib(2) == 2)
assert(fib(3) == 3)
assert(fib(4) == 5)
assert(fib(5) == 8)
assert(fib(6) == 13)
