cls = table
cls.type = 'Person'

person = table :: cls

print(cls.type)
print(person.type)
print('---')

cls.type = 'Imaginary Person'

print(cls.type)
print(person.type)
print('---')

person.type = 'Concrete Person'

print(cls.type)
print(person.type)
print('---')

(*person).type = 'Abstract Person'

print(cls.type)
print(person.type)
print('---')
