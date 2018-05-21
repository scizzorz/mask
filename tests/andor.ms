assert(('one' and 'two' or false) == 'two')
assert((null and 'two' or 'three') == null)
assert(('one' and null or 'three') == 'three')
assert(('one' and 'two' or null) == 'two')
