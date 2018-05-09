cls = table
cls.type = 'Person'

person = table :: cls
person.name = 'John'

print 'cls:'
print cls
print cls.type
print cls.name

print ''
print 'person:'
print person
print person.type
print person.name

print ''
print 'deref:'
print *cls
print *person
print (*person).type
print (*person).name
