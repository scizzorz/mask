i = 0
while i < 10
  print i
  i = i + 1

print '---'
loop
  i = i + 1

  if i < 20
    continue

  if i == 100
    break

  if i < 20 or 30 < i < 35
    print i

