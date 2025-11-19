# Decorators

Decorators in lulu operate like separate trait applicator functions that apply onto different entities such as:
- **classes**: with `class!` or `{..} -> `
- **class methods**: with `class!` or `() => ` or `function a:b()`
- **functions**: with `() => ` or `function()`
- **enums**: with `enum!` or `{...} -< `
- **enum variants**: with `enum!`
- **function/class/method params**: with `class!`, `() =>` or `function()`

## Decorator types

Based on the entity a decorator is applied to, it's calls switch in order and functionality.

### Class Decorators

The first support for decorators is on classes, class decorators take the class and should return a class back for other decorators to use. 

```lua
function MyClassDeco(_class, name)
  -- You can manipulate the class here
  return _class
end

-- Apply the decorator
class! @MyClassDeco MyClass;
```

Usually, you'd use class decorators to apply methods and properties through `init`.

```lua
function MyClassDeco(_class, name)

  -- This is how you would apply properties
  function _class:init(arg1, arg2)
    self.some_prop = arg1
  end

  -- This is how you would apply methods
  function _class:some_function(a, b)
    print(a, b)
    return self
  end

  return _class
end
```

### Class Method Decorators

Class Methods can have decorators mainly through the `class!` macro.

```lua

function MyMethodDeco(_class, _func, name)
  -- Takes a class and a function, and should return a function.
  return function(self, arg1, arg2)
    return _func(self, arg1, arg2)
  end
end

class! MyClass, {
  @MyMethodDeco
  something(arg1, arg2){

  }
}
-- Or

@MyMethodDeco
function MyClass:something(arg1, arg2)

end

-- Or

(arg1, arg2) @MyMethodDeco MyClass:something =>

end
```

### Function Decorators

Function decorators are a exactly like class method decorators but without the classes.

```lua
function MyFuncDeco(_func, name)
  -- Takes a function, and should return a function.
  return function(self, arg1, arg2)
    return _func(self, arg1, arg2)
  end
end

@MyFuncDeco
function MyFunction(arg1, arg2)

end

-- Or

(arg1, arg2) @MyFuncDeco MyFunction =>

end
```

### Enum Decorators

Enum decorators, just like Class decorators, 