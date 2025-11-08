--- TODO ---

type myType = string or number 

type person = {
  name = string,
  id = number
}

-- `for` because : would be weird here
-- `for` in here basically means
-- `as` or `:` in typescript
local myvar for myType = "something"
local myvar2 for person = {
  name = "sss",
  id = 0
} -- works

-- typed functions
function somefn(name: string, age: number, aquaintance: table<number,person>) -> person
  return ...
end

-- type generics, `in` is like `extends` in typescript.
function myfn<t in myclass = myclass>(name: t) -> t
  return name
end

myfn(myclass_instance)
myfn(class_that_extends_myclass_instance)
myfn<class_that_extends_myclass>(class_that_extends_myclass_instance)

-- function types
function mysomefn(func: fun(string) -> string) -> string
  return func("something")
end