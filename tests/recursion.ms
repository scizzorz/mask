fact = fn(n)
  if n == 1
    return 1
  return n * fact(n - 1)

fib = fn(n)
  if n == 0 or n == 1
    return 1
  return fib(n - 1) + fib(n - 2)

print fact(1)
print fact(2)
print fact(3)
print fact(4)
print fact(5)
print '---'
print fib(0)
print fib(1)
print fib(2)
print fib(3)
print fib(4)
print fib(5)
print fib(6)
