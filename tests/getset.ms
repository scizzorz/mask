f = 10
assert(f == 10)
set(local, 'f', 20)
assert(f == 20)
local:set('f', 30)
assert(f == 30)

f = 10
assert(f == 10)
f = 20
assert(get(local, 'f') == 20)
f = 30
assert(local:get('f') == 30)
