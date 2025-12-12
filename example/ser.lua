
using {
  lulib.serde
}


local {
  Admin,
  User(username),
} -< @Serializable('json') Role

local {
  #id,
  #content
} -> @Serializable('json') Post

local {
  #city
} -> @Serializable('json') Address

local {
  #name,
  #id,
  @Deserializable(Address)
  #address,
  @Deserializable(Role)
  #role,
  @Deserializable(Vec:of(Post))
  #posts
} -> @Serializable('json') User

local user = User {
  name = "name",
  id = 10,
  address = {
    city = "dd"
  },
  role = Role.User("somedude"),
  posts = vec!{
    Post {
      id = 1,
      content = "ddd"
    }
  }
}

fprint(User:deserialize(tostring(user)))
-- fprint(tostring(Role.Admin))
-- fprint(tostring(user))
