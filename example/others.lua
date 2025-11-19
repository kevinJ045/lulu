using {
  lulib.serde
}


local {
  @default_to("name")
  #name
  @default_to(0)
  #id
} -> @Serializable('yaml') Struct

fprint(Struct({ name = "jdjd" }):serialize())

local f = Struct { name = "TheName" }

local object = {
  some_name = "sjsjs"
}
() @namespace(object) =>
  print(some_name)
  jd = "ssl"
end
print(object.jd)