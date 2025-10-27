
using { lulib.net, lulib.kvdb }

() @namespace(Serve) =>

  local {
    #id,
    #by,
    #title,
    #content
  } -> @Serializable('json') @DBTable('id', 'title', 'by') UserPost

  local {
    #id,
    #name,
    #token,
    #age,
  } -> @Serializable('json') @DBTable('name', 'token') User

  local db = kvdb.open('proj/.lib/db')

  local users = kvdb.table_of(db, 'users', User)
  local posts = kvdb.table_of(db, 'posts', UserPost)

  (req) @Guard AuthGuard =>
    local token = req.headers['authorization']
    if not token then
      return error_res! 401, "Missing authorization token.";
    end

    local found = users:find('token', token):into()
    if found.len() < 1 then
      return error_res! 403, "Invalid token.";
    end

    req.user = found.get(1)
    return true
  end

  local {} -> @Controller("/users") Users

  (@Param('name') name, @Query('select') selected) @Get("/:name/posts") @Serialized Users:getPosts =>
    return posts:find('by', name):select(selected or  'id,title,content')
  end

  (@Param('name') name, @Query('select') selected) @Get("/:name") @Serialized Users:getUser =>
    local user = users:find('name', name):into()
    if user.len() < 1 then
      return error_res! 404, f"User {name} not found.";
    end
    return user.select(selected or 'name, age'):get(1)
  end

  (@Body(User) user) @Post("/") Users:setUser =>
    user:into()
    return json_res! {{ done = true }}
  end

  local {} -> @Controller("/posts") @UseGuard(AuthGuard) Posts

  (@Body(UserPost) post, @Context('user') user) @Post("/") Posts:createPost =>
    post.by = user.name
    post:into()
    return json_res! {{ success = true, author = user.name }}
  end
  
  (@Body(UserPost) new_post, @Context('user') user, @Param('id') id) @Put("/:id") Posts:editPost =>
    local post = posts:index(id)

    if not post then
      return error_res! 404, f"Post {id} not found.";
    end

    if post.by != user.name then
      return error_res! 401, f"Not authorized.";
    end

    for k, v in pairs(new_post) do
      post[k] = v
    end

    posts:update(id, post)
    return json_res! {{ success = true, author = user.name }}
  end

  (@Context('user') user, @Param('id') id) @Delete("/:id") Posts:deletePost =>
    local post = posts:index(id)

    if not post then
      return error_res! 404, f"Post {id} not found.";
    end

    if post.by != user.name then
      return error_res! 401, f"Not authorized.";
    end

    posts:remove(id)
    return json_res! {{ success = true, author = user.name }}
  end

  Server("0.0.0.0:8000")
    :use(Users)
    :use(Posts)
    :start()
    
end

test! {
  create_and_use_user {
    () async =>
      net.http.post("http://localhost:8000/users/", {
        body = serde.json.encode({
          name = "dkkd",
          age = 3093
        })
      })

      fprint(net.http.get("http://localhost:8000/users/dkkd").text())
    end
  }
}