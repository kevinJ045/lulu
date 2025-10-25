# `class!`

> Generating Macro

The `class!` macro introduces an expressive syntax for creating classes in Lua, handling all the metatable boilerplate for you. It kinda partially supports constructors, methods, inheritance, and even decorators.

## Basic Class

Let's first tackle the basics, not too much, not too little.

```lua
-- To make a class, we simply only need to call the
-- class macro with the name and empty block
local class! Person, {}

-- Now, let's start with an initiator
class! Person, {
  -- init is called at the construction
  -- of the class.
  init() {}
  -- there can be multiple init functions,
  -- and they are all together called with
  -- __call_init.
}

-- Now, let's add a few attributes and methods to our class
class! Person, {
  init(name, age) {
    self.name = name
    self.age
  }

  greet() {
    print(f"Hello, my name is {self.name}.")
  }
}

-- While we can use init to manually manage
-- param-data cases, we can also do this to simplify
-- everything
class! Person(name, age), {
  greet() {
    print(f"Hello, my name is {self.name}.")
  }
}

-- To use the Person class we simply do:
local p = Person("Tim", 30)
p:greet()
```

## Constructors

Constructors let you change the way a whole class initiates. Constructors run before `init`.

```lua

-- We can use the constructor as the code that follows.
-- We can use it to custom set the parameters
-- if they need to be parsed or alike.
class! Person, (name, age) {
  self.name = String(name)
  self.age = age or 0

  -- whatever else you want here
}, {
  greet() {
    print(f"Hello, my name is {self.name}.")
  }
}

-- Skipping parameters with constructors:
class! Person(name), (_, age) {
  self.age = age or 0
}, {
  greet() {
    print(f"Hello, my name is {self.name}.")
  }
}

-- You can still access variables that have been
-- passed to the class name definition.
class! Person(name), (name, age) {
  iself.name = String(name)
  self.age = age or 0
}, {
  greet() {
    print(f"Hello, my name is {self.name}.")
  }
}
```

## Inheritance

Keeping it simple, let's take a look at inheritance,

```lua

class! Animal(name), {
  walk() {
    print(f"{self.name} is walking.")
  }
}

-- When you inherit, you get all the functions
-- from the parent, and the (name) parameter
-- for example is parsed by default
class! Cat:Animal, {}

-- If you specify the parameters you HAVE to respect
-- the parent's paramets and either include them, 
-- or just skip with _
class! Cat:Animal(name, voice), {}
-- or
class! Cat:Animal(_, voice), {}


-- Full class with inheritance:
class! Cat:Animal(name, voice), {
  speak() {
    print(self.voice)
  }
}

-- You can use normally as you
-- would a normal class:
local my_cat = Cat("Whiskers", "Meow!")
my_cat:walk()  -- Output: Whiskers is walking.
my_cat:speak() -- Output: Meow!

```

## Inheritance and constructors

A little complex but still somewhat simple.

```lua

-- By default, the parent constructor is called
-- before the child constructor. for example:
-- But by using super we can manage how the parent is called.
class! Cat:Animal, (name, voice) {
  super(name)
  self.voice = voice
}, {
  speak() {
    print(self.voice)
  }
}

-- the super function only exists in the constructor call,
-- and it is called by default inside the constructor.
-- However, as soon as you use it, the class macro
-- will let you call it manually and ejects
-- the automatic super call code from generating.

```

## Named attributes

Instead of using indexes for attributes, you can use `#` in the parameter name to make it object based

```lua

class! Person(#name, #id);

-- Call with:
local person = Person {
  name = "John",
  id = 0
}

-- You can also do multiple values as:

class! Person(#name, #id, _, #city);

-- And call with:

local person = Person({
  name = "John",
  id = 0
}, {
  city = "my-city"
})

```

## Decorators

The class! macro also supports experimental decorators for classes, methods, and parameters, allowing for metaprogramming.

- **Class Decorators**: Applied to the class itself.
- **Method Decorators**: Applied to a method within a class.
- **Parameter Decorators**: Applied to the arguments of a method.

```lua

-- Class decorators:

-- Decorators are just functions that take 
-- the class and manipulate it
function speaks(class) -- takes the class
  function class:init(_, voice) -- staggers to the inits
    self.voice = voice
  end

  function class:speak() -- 
    print(self.voice)
  end

  return class
end

-- Apply the decorator with it's set of class definitions.
class! @speaks Cat:Animal, {}

local c = Cat("Whiskers", "Meow!")
c:speak() -- Meow!
c:walk()

-- You can also have multiple decorators
-- Keep in mind, the order is from
-- last to first
function loud_speak(class)
  local old_speak = class.speak
  function class:speak()
    io.write("!!! ")
    old_speak(self)
  end
  return class
end

class! @loud_speak @speaks Cat:Animal, {}


-- Double class decorators:

-- In here, we have a function
-- that builds the decorator
-- based on parameters
function speaks(voice) -- custom attributes
  -- this is the actual decorator
  return function(class)
    function class:speak() 
      print(voice)
    end

    return class
  end
end

class! @speaks("Meow!") Cat:Animal, {}


-- Method decorators:

-- Method decorators take the class and default method,
-- and they need to return a function
function uppercase_name(class, method)
  return function(self, ...)
    local old_name = self.name
    self.name = self.name:upper()
    method(self, ...)
    seld.name = old_name
  end
end

class! Person(name, age), {
  @uppercase_name
  greet() {
    print(f"Hello, my name is {self.name}.")
  }
}

-- Parameter decorators:

-- A parameter decorator receives (self, raw_param_value)
-- and returns the transformed value.
function default_age(self, value)
  return value or 18
end

class! Person(name, age), {
  init(name, @default_age age) {
    self.name = name
    self.age = age
  }

  greet() {
    print(f"{self.name} is {self.age} years old.")
  }
}

```
