using {
  lulib.serde
}

@trait({
  ddd = function()end
})
function Ddd() end

@trait({
  name = {default_to(''), map_into(function(val) return f"{val}-cat" end), validate_type("string")},
  meow = function(self)
    print(f"{self.name} is meowing")
  end
})
function Cat(self)
  self.is_cat = true
end

@derive({
  id = 0
}, Cat, Serializable('json'))
function MyClass(self)

end

function MyClass:say()
  print(self.name)
end

function MyClass::new(a)
  return { name = a }
end

local x = MyClass { name = '33' }

x.say()

print(x.is_cat)
x.meow()

print('satiates', derive.satiates(x, Cat, Ddd))

fprint(x.serialize())
-- fprint(MyClass::deserialize([[{"id":22,"is_cat":true,"name":"a"}]]))
-- fprint(x.clone())

-- fprint(MyClass::new('s'))

class! @with_trait(Cat) SomeClass(@default_to('a') #a, #b, #name);

local y = SomeClass { name = "ff" }

y:meow()