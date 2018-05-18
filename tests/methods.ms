int_mt = table
int_mt.double = |x| x * 2

six = 6 :: int_mt
f = fn()
  print('evaluating lhs')
  return 5 :: int_mt

print(six)
print(six.double)
print(six.double(six))
print(six:double())
print(f():double())
