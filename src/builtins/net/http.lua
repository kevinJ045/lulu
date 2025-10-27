
local class! HttpMetadata(
  @default_to({})
  #headers,
  #body,
  @default_to("")
  #url,
  @default_to("")
  #uri,
  @default_to({})
  #query,
  @default_to("GET")
  #method,
), {
  init(){
    if self.uri:find("?") then
      local query_str = self.uri:match("%?(.*)$")
      for key, val in query_str:gmatch("([^&=?]+)=([^&=?]+)") do
        self.query[key] = val
      end
    end
  }
  json(){
    self.headers["Content-Type"] = "application/json"
    return self
  }
  is_json(){
    return self.headers["Content-Type"] == "application/json"
  }
};

class! Request:HttpMetadata;
class! Response:HttpMetadata(
  @default_to(200)
  #status,
);


local function _handle_response(response)
  local body = response.body
  local headers = response.headers or {}
  local status = response.status or 0

  local res = {
    status = status,
    headers = headers,
  }

  res.text = function()
    if type(body) == "string" then
      return body
    elseif type(body) == "userdata" then
      return body:to_string()
    end
    return tostring(body)
  end

  res.json = function()
    local text = res.text()
    local ok, decoded = pcall(serde.json.decode, text)
    if not ok then
      error("Invalid JSON: " .. tostring(decoded))
    end
    return decoded
  end

  res.into_many = function(_class)
    local params = {}
    try_catch! {
      params = res.json()
    }, {
      try_catch! {
        params = res.yaml()
      }, {
        params = {}
      }
    }

    local items = Vec()
    for k, v in pairs(params) do
      items:push(_class(v))
    end

    return items
  end

  res.into = function(_class)
    if not _class then return end

    if _class.deserialize then
      return _class:deserialize(res.text())
    else
      local params = {}
      try_catch! {
        params = res.json()
      }, {
        try_catch! {
          params = res.yaml()
        }, {
          params = {}
        }
      }

      return _class(params)
    end
  end

  res.yaml = function()
    local text = res.text()
    local ok, decoded = pcall(serde.yaml.decode, text)
    if not ok then
      error("Invalid YAML: " .. tostring(decoded))
    end
    return decoded
  end

  res.body = function()
    return body
  end

  return res
end

macro {
  ____request_method($method, $method_string){
    net.http.$method = function(url, options)
      local data = options or {}
      local response = net.http.request {
        method = data.method or $method_string,
        url = url,
        headers = data.headers or {},
        body = data.body,
      }
      return _handle_response(response)
    end
  }
}

____request_method! send, "GET";
____request_method! get, "GET";
____request_method! post, "POST";
____request_method! patch, "PATCH";
____request_method! put, "PUT";
____request_method! delete, "DELETE";

Serve = {}

--------------------------------------------------
-- Utilities
--------------------------------------------------
local function join_path(a, b)
  if a:sub(-1) == "/" then a = a:sub(1, -2) end
  if b:sub(1, 1) == "/" then b = b:sub(2) end
  return a .. "/" .. b
end

local function clone(tbl)
  local t = {}
  for k, v in pairs(tbl) do
    t[k] = v
  end
  return t
end

local function match_path(template, actual)
  local params = {}

  -- Escape Lua pattern magic characters
  local escaped = template:gsub("([%^%$%(%)%%%.%[%]%*%+%-%?])", "%%%1")

  -- Replace :param placeholders with capture groups
  local pattern = escaped:gsub(":(%w+)", function(pname)
    params[#params+1] = pname
    return "([^/]+)"
  end)

  -- Allow optional trailing slash in both template and actual
  pattern = pattern:gsub("/+$", "") .. "/?$"

  -- Try to match
  local captures = { actual:match("^" .. pattern) }

  -- No match? fail fast
  if #captures == 0 then
    return false
  end

  -- Build param map
  local map = {}
  for i, pname in ipairs(params) do
    map[pname] = captures[i]
  end

  return true, map
end

local function clean_uri(uri)
  return String(uri):split('?'):get(1)
end

function Serve.Server(addr, fn)
  local server = {
    __addr = addr,
    __controllers = {},
    __services = {},
    __middlewares = {},
  }

  function server:use(controller)
    -- Instantiate the controller with injected services
    if controller.__call_init then
      local instance = controller(self.__services)
      table.insert(self.__controllers, instance)
    elseif controller then
      table.insert(self.__middlewares, controller)
    end
    return server
  end

  function server:provide(service)
    local meta = getmetatable(service)
    if meta and meta.__is_service then
      self.__services[meta.__name] = service()
    end
    return server
  end

  function server:start()
    net.http.serve(self.__addr, function(req)
      return server:_handle(req)
    end)
    return server
  end

  function server:_handle(req)
    for _, ctrl in ipairs(self.__controllers) do
      local cmeta = ctrl.controller
      for _, route in ipairs(cmeta.__routes) do
        local matched, params = match_path(route.path, clean_uri(req.uri))
        if route.method == req.method and matched then
          local req = Request(req)

          req.params = params
          
          for _, mw in ipairs(server.__middlewares) do
            local mr = mw:handle(req)
            if mr then
              return mr
            end
          end

          for _, guard in ipairs(cmeta.__guards) do
            local gr = guard.check(req)
            if not gr or instanceof(gr, Response) then
              return gr or { body = "Unauthorized", status = 401 }
            end
          end

          for _, guard in ipairs(route.guards or {}) do
            local gr = guard.check(req)
            if not gr or instanceof(gr, Response) then
              return gr or { body = "Unauthorized", status = 401 }
            end
          end

          for _, mw in ipairs(cmeta.__middlewares) do
            local mr = mw:handle(req)
            if mr then
              return mr
            end
          end
          for _, mw in ipairs(route.middlewares or {}) do
            local mr = mw:handle(req)
            if mr then
              return mr
            end
          end

          ctrl.req = req
          local result = route.handler(ctrl)
          ctrl.req = nil
          local body = result
          local headers = {}
          
          if instanceof(result, Response) then
            return result
          end

          return {
            body = body,
            headers = {},
            status = 200
          }
        end
      end
    end

    return { body = "Not Found", status = 404 }
  end

  if fn then fn(server) end
  return server
end

function Serve.Controller(base_path)
  local meta = {
    __base = base_path or "",
    __routes = {},
    __middlewares = {},
    __guards = {}
  }

  return function(class)
    class.controller = meta
    return class
  end
end

function Serve.UseGuard(guard)
  return decorator! {
    (_class, method){
      for _, route in ipairs(_class.controller.__routes) do
        if route.handler == method then
          table.insert(route.guards, guard)
        end
      end
      return method
    }
    (_class){
      table.insert(_class.controller.__guards, guard)
      return _class
    }
  }
end

function Serve.UseMiddleware(mw)
  return decorator! {
    (_class, method){
      for _, route in ipairs(class.controller.__routes) do
        if route.handler == method then
          table.insert(route.middlewares, mw)
        end
      end
      return method
    }
    (_class){
      table.insert(_class.controller.__middlewares, mw)
      return _class
    }
  }
end

local function make_method_decorator(method)
  return function(path)
    return function(class, fn)
      table.insert(class.controller.__routes, {
        method = method,
        path = join_path(class.controller.__base, path),
        handler = fn,
        guards = {},
        middlewares = {},
      })
      return fn
    end
  end
end

Serve.Get = make_method_decorator("GET")
Serve.Post = make_method_decorator("POST")
Serve.Put = make_method_decorator("PUT")
Serve.Patch = make_method_decorator("PATCH")
Serve.Delete = make_method_decorator("DELETE")

function Serve.Service(class, name)
  setmetatable(class, {
    __is_service = true,
    __name = name or tostring(class)
  })
  return class
end

function Serve.Guard(fn)
  return  {
    __type = "guard",
    check = fn
  }
end

function Serve.Middleware(class, fn)
  return  {
    __type = "middleware",
    handle = fn
  }
end

function Serve.Param(name)
  return function(self)
    return self.req.params[name]
  end
end

function Serve.Query(name)
  return function(self)
    return self.req.query[name]
  end
end

function Serve.Context(name)
  return function(self)
    return self.req[name]
  end
end

function Serve.Body(Class)
  return function(self)
    if Class then
      return Class:deserialize(self.req.body:to_string())
    else
      return self.req.body
    end
  end
end

function Serve.Serialized(class, method)
  return function(self)
    local result = method(self)
    if instanceof(result, Response) then
      return result
    end
    if not result.serialize then
      print("Class is not serializable")
    end
    return result:serialize()
  end
end
