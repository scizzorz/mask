cls = table()
cls.type = 'Person'

person = table() :: cls

assert(cls.type == 'Person')
assert(person.type == 'Person')

cls.type = 'Imaginary Person'

assert(cls.type == 'Imaginary Person')
assert(person.type == 'Imaginary Person')

person.type = 'Concrete Person'

assert(cls.type == 'Imaginary Person')
assert(person.type == 'Concrete Person')

(*person).type = 'Abstract Person'

assert(cls.type == 'Abstract Person')
assert(person.type == 'Concrete Person')
