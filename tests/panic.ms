f = fn()
  panic 5

print 'panic: expecting true'
x = catch
  panic true
print x

print 'panic: expecting 5'
x = catch
  f()
print x
