f = fn(x, y)
  print(x)
  print(y)

assert(x == null)
assert(f.f == f)
assert(f.x == null)
f.f = 'nope'
f.x = 'injexted'

assert(x == null)
assert(f.f == 'nope')
assert(f.x == 'injexted')

# this needs to be double-indirected because single indirection is implicit;
# ie, (f.x) aliases to (*f).x
(**f).x = 'injected'
(**f).f = 'huzzah'
assert(x == 'injected')
assert(f == 'huzzah')
assert(f.f == null)
