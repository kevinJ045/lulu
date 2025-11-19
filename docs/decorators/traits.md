# Derive and Traits

On top of decorators, we have simple little sets of attributes called traits. 

## Derive

With `derive`, you can make a quick function that behaves as a class.

```lua
@derive({
  -- default values
  name = "",
  id = 0
})
local function Person(self)

end

-- Instantiate with:
local person = Person {
  name = "John",
  age = 20
}

print(person.name, person.age)
```

But the main use of `derive` is to apply traits. You can apply many traits as such:

```lua

@derive({
  name = "",
  id = 0
}, TraitA, TraitB)
local function Person(self)

end
```

> You can read more about creating traits below

You can also use decorators for the values here.

```lua

@derive({
  name = { default_to(""), validate_type("string") },
  id = 0
})
local function Person(self)

end

```

## Traits

Traits are a simple set of functions and attributes that can be applied onto either classes or can be derived into a derive instance.

```lua
-- Traits are used just like `derive`
@trait({
  -- You can use decorators here too
  name = { default_to(""), validate_type("string") },
  id = 0
})
local function Person(self) end

--- Using in derive
@derive({
  age = 0
}, Person)
local function AgedPerson(self) end
```

### Trait inheritance

Traits can make a chain of traits to be applied all together.

```lua
-- This will work the same as `Person` but now it has `age`
@trait({
  age = 0
}, Person)
local function AgedPerson(self) end
```

### Trait methods

To make a trait method, you only need to make functions.

```lua
@trait({
  name = "",
  id = 0,
  speak = function(self)
    print(f"Hello, I am {self.name}")
  end 
})
local function Person(self) end

-- Use as:
local person = Person {
  name = "John",
  age = 20
}

person.speak()
```

### Checking trait validity

You can check if a trait is applied with:

```lua
derive.satiates(person, Person) -- true
```

### Traits in classes

To apply traits into classes, you can use the `with_trait` decorator.

```lua
-- keep in mind, you have to include the attributes of the
-- trait as such.
class! @with_trait(Person) Someone(#name, #id);

local {
  -- these are necessary
  #name,
  #id
} => @with_trait(Person) Someone
```