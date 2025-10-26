
using { lulib.net, lulib.serde }

-- using.lulib { export2, "./export.lua" }
-- net.keepalive()

async(function()
  local f = net.http.send("http://localhost:8080/")
  fprint(f.text())
  fprint(f.into_many(MyClass):get(1).others:get(1):get_name())
end)

local t = {}
class! @Serializable('json') MyClass(#name, #id, @Deserializable(t) #others), {
  get_name(){
    return self.name
  }
};
t = Vec:of(MyClass)

net.http.serve("0.0.0.0:8080", function(req)
  return {
    body = Vec({
      MyClass({
        name = "sjsjs",
        id = 838,
        others = Vec({
          MyClass({
            name = "dnccj",
            id = 434,
          })
        })
      })
    }):serialize()
  }
end)