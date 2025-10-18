

class! Person, {
  init(name, age) {
    self.name = name
    self.age = age
  }

  greet() {
    print(f"Hello, my name is {self.name} and i am {self.age} years old.")
  }
}

function Person:init()
  print(self.name)
end

local p = Person("John", 30)
p:greet()


function speaks(voice) -- custom attributes
  -- this is the actual decorator
  return function(class)
    function class:speak() 
      print(voice)
    end

    return class
  end
end

function loud_speak(class)
  local old_speak = class.speak
  function class:speak()
    io.write("!!! ")
    old_speak(self)
  end
  return class
end

class! Animal(name), {
  walk() {
    print(f"{self.name} is walking.")
  }
}

class! @into_collectible("collect") @loud_speak @speaks("Meow!") Cat:Animal, {}

local c = Cat("Whi"):into()
c.walk()
c.speak()
