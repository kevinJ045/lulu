# `class!`

> Generating Macro

The `class!` macro introduces an expressive syntax for creating classes in Lua, handling all the metatable boilerplate for you. It kinda partially supports constructors, methods, inheritance, and even decorators.

## Basic Class

Define a class with a name and a block of methods.

```lua
class! Person, {
  -- An `init` method serves as the constructor
  init(name, age) {
    self.name = name
    self.age = age
  },

  greet() {
    print(f"Hello, my name is {self.name}.")
  }
}

local p = Person("John", 30)
p:greet() -- Output: Hello, my name is John.
```

## Inheritance

You can inherit from a parent class using the `:` syntax.

```lua
class! Animal(name), {
  walk() {
    print(f"{self.name} is walking.")
  }
}

-- Cat inherits from Animal
class! Cat:Animal(name, voice), {
  speak() {
    print(self.voice)
  }
}

local my_cat = Cat("Whiskers", "Meow!")
my_cat:walk()  -- Output: Whiskers is walking.
my_cat:speak() -- Output: Meow!
```

## Decorators

The `class!` macro also supports experimental decorators for classes, methods, and parameters, allowing for powerful metaprogramming.

- **Class Decorators**: Applied to the class itself.
- **Method Decorators**: Applied to a method within a class.
- **Parameter Decorators**: Applied to the arguments of a method.

```lua
-- A hypothetical example of decorators

class!
@Singleton()
@Logger("INFO")
MyService,
{
  @TimeIt()
  some_task(@Validated("string") input) {
    -- ... implementation ...
  }
}
```

**Note**: Decorators are an advanced, experimental feature. A decorator is a function that receives the class or method as input and is expected to return a modified version.
