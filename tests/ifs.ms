f = fn(n)
  if n < 0
    assert n < 0
  else
    assert n >= 0

  if n < 2
    assert n < 2
  else if n < 4
    assert 2 <= n < 4
  else if 6 < n < 8
    assert 6 < n < 8
  else
    assert (4 <= n <= 6) or (n >= 8)


i = 0 - 3
while i < 10
  f(i)
  i = i + 1
