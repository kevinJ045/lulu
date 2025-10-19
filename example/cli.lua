
class! Classic(@validate_type("string") #name, @validate_type("number") #id);

local c = Classic {
  name = "ff",
  id = 33
}
fprint(c)
