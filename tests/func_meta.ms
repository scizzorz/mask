f = fn(x, y)
  print(x)
  print(y)

print x
print f
print f.f
print f.x
print '---'
(*f).f = 'nope'
(*f).x = 'injexted'

print x
print f
print f.f
print f.x
print '---'

(**f).x = 'injected'
(**f).f = 'huzzah'
print x
print f
print f.f
print f.x
print '---'
