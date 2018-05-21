i = 0
sum = 0
while i < 10
  sum = sum + i
  i = i + 1

assert(sum == 45)

loop
  i = i + 1

  if i < 20
    continue

  if i == 100
    break

  if i < 20 or 30 < i < 35
    sum = sum + i

assert(sum == 175)
