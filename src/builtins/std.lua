
macro {
  vec ($block) {Vec({ $block }):into()}
}

function dump_item_into_string(o, indent)
  indent = indent or 0
  if type(o) == 'table' then
    local s = '{\n'
    for k, v in pairs(o) do
      if type(k) == "number" or k:sub(1, 2) ~= "__" then
        s = s .. string.rep('  ', indent + 1) .. tostring(k) .. ' = ' .. dump_item_into_string(v, indent + 1) .. ',\n'
      end        
    end
    return s .. string.rep('  ', indent) .. '}'
  else
    return tostring(o)
  end
end

function fprint(...)
  local args = {}
  for key, item in ipairs({...}) do
    if item == nil then
      args[key] = 'nil'
    else
      args[key] = dump_item_into_string(item)
    end
  end
  print(unpack(args))
end

function namespace(tbl)
  return function(chunk)
    chunk = chunk or function() end
    setfenv(chunk, setmetatable(tbl or {}, { __index = _G }))
    return chunk(tbl)
  end
end

function make_enum(name)
  local e = {}
  e.func = {}

  function e.is(obj, variant)
    if type(obj) ~= 'table' then return false end
    if obj.__enum == nil then return false end
    if variant then
      if obj.__enum_var == nil then return false end
      return obj.__enum_var == variant or obj.__enum_var == variant.__enum_var
    end
    return obj.__enum == e
  end
  
  e.__is_enum = true
  e.__name = name or ""
  
  return e
end

local __enum_var_name = {}
function make_enum_var_dyn(enum_table, vname, names)
  return function(...)
    local args = {...}
    if args[1] == __enum_var_name then
      return vname
    end
    return make_enum_var(enum_table, vname, names, ...)
  end
end

function get_enum_var_name(var)
  if type(var) == "function" then
    return var(__enum_var_name)
  else
    return var.__enum_var_name
  end
end

function index_of(object, item)
  for i, val in ipairs(object) do
    if val == item then return i end
  end
end

function make_enum_var(enum_table, vname, names, ...)
  local o = {}
  local args = {...}
  for i, arg in ipairs(args) do
    o[names[i] or i] = arg
  end
  o.__enum = enum_table
  o.__enum_var_name = vname
  o.__enum_var = enum_table[vname] or vname
  o.__is = function(b)
    if type(b) == 'function' then return o.__enum_var == b end
    if type(b) == 'table' and b.__enum_var then return o.__enum_var == b.__enum_var end
    if type(b) == 'table' and b.__is_enum then return o.__enum == b end
    return o.__enum_var == b or o == b
  end
  setmetatable(o, {
    __index = function(tbl, key)
      local item = enum_table.func[key]
      if type(item) == 'function' then
        return function(...) return item(o, ...) end
      end
      return item
    end
  })
  return o
end

function make_class(class_raw, parent)
  local class = class_raw
  class.__index = class
  local inits = {}

  function class:__call_init(...)
    if parent and parent.__call_init then
      parent.__call_init(self, ...)
    end
    for _, fn in ipairs(inits) do
      fn(self, ...)
    end
  end

  local class_meta = {
    __call = function(cls, ...)
      local self = setmetatable({}, cls)
      self.__class = cls
      if self.__construct then self:__construct(true, ...) end
      return self
    end,

    __newindex = function(t, k, v)
      if k == "init" and type(v) == "function" then
        table.insert(inits, v)
      else
        rawset(t, k, v)
      end
    end,
  }

  if parent then
    class_meta.__index = parent
    class_meta.__parent = parent
  end

  setmetatable(class, class_meta)
  
  return class
end

function iseq(first, second)
  local result = first == second
  if result then return result end

  if first and type(first) == "table" and first.__is and first.__is(second) then
    return true
  end

  if first and type(first) == "table" and type(second) == "table" and second.is and second.is(first) then
    return true
  end
  
  if first and type(first) == "table" and instanceof(first, second) then
    return true
  end

  return result
end

function empty_class()
  return {
    __class = {
      empty = true
    }
  }
end

function instanceof(obj, class)
  if not obj then return false end
  if not class and obj then return false end
  local cls = obj.__class
  while cls do
    if cls == class then
      return true
    end
    cls = getmetatable(cls) and getmetatable(cls).__parent
  end
  return false
end

__future_stack = {}

Future = {}
Future.__index = Future

function Future.new(fn)
  local self = setmetatable({}, Future)
  self.co = coroutine.create(fn)
  self.done = false
  self.result = nil
  self.error = nil
  self.onError = function(e)
    error(e)
  end
  self.onAfter = function(e)
    return e
  end
  table.insert(__future_stack, self)
  return self
end

function Future:poll(...)
  if self.done then return self.result end
  local ok, res = coroutine.resume(self.co, ...)
  if not ok then
    self.error = res
    self.done = true
    return
  end
  if coroutine.status(self.co) == "dead" then
    self.done = true
    self.result = res
  else
    -- Yield control back to the scheduler after every poll
    coroutine.yield()
  end
  return res
end

function Future:last()
  if self.error then self.onError(self.error) end
  return self.onAfter(self.result)
end

function Future:await()
  while not self.done do
    self:poll()
  end
  return self:last()
end

function Future:after(cb)
  local olOnAfter = self.onAfter
  self.onAfter = function(r)
    return cb(olOnAfter(r))
  end
  return self
end

function Future:catch(cb)
  self.onError = cb
  return self
end

function async(fn)
  return Future.new(fn)
end

Future.scheduler = coroutine.create(function()
  local i = 1
  while #__future_stack > 0 do
    local fut = __future_stack[i]
    if not fut.done then 
      fut:poll()
    end
    if fut.done then
      fut:last()
      table.remove(__future_stack, i)
    else
      i = i + 1
    end
    if i > #__future_stack then i = 1 end
    coroutine.yield()
  end

  return false
end)



enum! Option, {
  Some(content),
  None
}

Some = Option.Some
None = Option.None

Option.func.unwrap = function(item)
  return item.content and item.content or nil
end

enum! Result, {
  Ok(content),
  Err(err)
}

Ok = Result.Ok
Err = Result.Err

Result.func.unwrap = function(item)
  return item.content and item.content or item.err
end

function into_collectible(name)
  return function(class)
    function class:into()
      local parent = self
      local proxy = {}

      proxy[name] = function()
        return self
      end

      function proxy.clone()
        return parent:clone():into()
      end

      setmetatable(proxy, {
        __index = function(_, key)
          local val = parent[key]

          if type(val) == "function" then
            return function(...)
              local result = val(parent, ...)
              if result == parent then
                return proxy
              else
                return result
              end
            end
          end

          return val
        end,
        __tostring = function()
          return parent:__tostring()
        end,
      })

      return proxy
    end

    return class
  end
end

function validate_type(...)
  local types = {...}

  return decorator! {
    _ {
      local verify = function(...)
        local args = {...}
        for i, arg in ipairs(args) do
          if types[i] == '!' then

          elseif type(arg) != types[i] and not iseq(arg, types[i]) then
            local t = types[i]
            if types[i] != "number" and types[i] != "string" then
              t = f"abstract({tostring(types[i])})"
            end
            error(types[i] and f"Expected {t} for {name} at argument {i}. Found {type(arg)}" or (
              #types > #args and f"Expected {#types} arguments for {name}, given {#args}" or f"Extra args for {name}."
            ))
          end
        end
        return args
      end
    }
    (_class, method) {
      return function(self, ...)
        return method(self, unpack(verify(...)))
      end
    }
    (_enum, variant) {
      dynamic {
        return function(...)
          return variant(unpack(verify(...)))
        end
      }
      static {
        return variant
      }
    }
    (_self, value) {
      return verify(value)[1]
    }
  }
end


class! @into_collectible("collect") Vec, {
  init(len) {
    if type(len) == "number" then
      self.items = {}
      for i = 1, len do
        self.items[i] = false
      end
    elseif type(len) == "table" then
      self.items = len
    else
      self.items = {}
    end
  }

  push(...) {
    local args = {...}
    for _, v in ipairs(args) do
      table.insert(self.items, v)
    end
    return self
  }

  pop(){
    return table.remove(self.items)
  }

  len(){
    return #self.items
  }

  get(index) {
    return self.items[index]
  }

  set(index, value) {
    self.items[index] = value
    return self
  }

  for_each(callback) {
    for i, v in ipairs(self.items) do
      callback(v, i, self)
    end
  }

  map(callback) {
    local result = {}
    for i, v in ipairs(self.items) do
      result[i] = callback(v, i, self)
    end
    return Vec(result)
  }

  filter(callback) {
    local result = {}
    for i, v in ipairs(self.items) do
      if callback(v, i, self) then
        table.insert(result, v)
      end
    end
    return Vec(result)
  }

  join(sep) {
    return table.concat(self.items, sep or ", ")
  }

  __tostring(){
    return "[" .. table.concat(self.items, ", ") .. "]"
  }

  find(fn) {
    for i, v in ipairs(self.items) do
      if fn(v, i, self) then
        return i, v
      end
    end
    return nil
  }

  remove_at(index) {
    table.remove(self.items, index)
    return self
  }

  remove(fn) {
    local new = {}
    for i, v in ipairs(self.items) do
      if not fn(v, i, self) then
        table.insert(new, v)
      end
    end
    self.items = new
    return self
  }

  insert(index, item) {
    table.insert(self.items, index, item)
    return self
  }

  extend(...) {
    local arrays = {...}
    for _, arr in ipairs(arrays) do
      if getmetatable(arr) == getmetatable(self) then
        for _, v in ipairs(arr.items) do
          table.insert(self.items, v)
        end
      elseif type(arr) == "table" then
        for _, v in ipairs(arr) do
          table.insert(self.items, v)
        end
      else
        table.insert(self.items, arr)
      end
    end
    return self
  }

  reverse(){
    local len = #self.items
    for i = 1, math.floor(len / 2) do
      local j = len - i + 1
      self.items[i], self.items[j] = self.items[j], self.items[i]
    end
    return self
  }

  sort(fn) {
    if fn then
      table.sort(self.items, fn)
    else
      table.sort(self.items)
    end
    return self
  }

  clone(){
    return Vec({unpack(self.items)})
  }

}

class! @into_collectible("to_string") String, {
  init(s){
    if type(s) == "string" then
      self.str = s
    else
      self.str = ""
    end
  }

  push_str(s){
    self.str = self.str .. tostring(s)
    return self
  }

  push_string(other){
    if getmetatable(other) == String then
      self.str = self.str .. other.str
    else
      self:push_str(other)
    end
    return self
  }

  split(sep){
    local t = {}
    sep = sep or "%s"
    for part in self.str:gmatch("([^" .. sep .. "]+)") do
      table.insert(t, part)
    end
    return Vec(t)
  }

  starts_with(prefix){
    return self.str:sub(1, #prefix) == prefix
  }

  ends_with(suffix){
    return self.str:sub(-#suffix) == suffix
  }

  match(pattern){
    return re.match(pattern, self.str)
  }

  replace(pattern, repl){
    self.str = re.replace(pattern, self.str, repl)
    return self
  }

  as_str(){
    return self.str
  }

  clone(){
    return String("" .. self.str)
  }

  __tostring(){
    return self.str
  }
}

class! @into_collectible("collect") Set, {
  init(items){
    self.items = {}
    if type(items) == "table" then
      for _, v in ipairs(items) do
        self.items[v] = true
      end
    end
  }

  add(value) {
    self.items[value] = true
    return self
  }

  remove(value) {
    self.items[value] = nil
    return self
  }

  has(value) {
    return self.items[value] ~= nil
  }

  clear() {
    self.items = {}
    return self
  }

  values(){
    local vals = {}
    for k, _ in pairs(self.items) do
      table.insert(vals, k)
    end
    return Vec(vals)
  }

  clone(){
    local copy = Set()
    for k, _ in pairs(self.items) do
      copy:add(k)
    end
    return copy
  }
}

class! WeakSet:Set, {
  init(){
    self.items = setmetatable({}, { __mode = "k" })
  }
}


class! @into_collectible("collect") Map, {
  init(items){
    self.items = {}
  }

  set(key, value){
    self.items[key] = value
    return self
  }

  get(key, default){
    local v = self.items[key]
    if v == nil then
      return default
    else
      return v
    end
  }

  has(key){
    return self.items[key] ~= nil
  }

  remove(key){
    self.items[key] = nil
    return self
  }

  keys(){
    local keys = {}
    for k, _ in pairs(self.items) do
      table.insert(keys, k)
    end
    return Vec(keys)
  }

  values(){
    local vals = {}
    for _, v in pairs(self.items) do
      table.insert(vals, v)
    end
    return Vec(vals)
  }

  clone(){
    local copy = Map()
    for k, v in pairs(self.items) do
      copy:set(k, v)
    end
    return copy
  }
}


class! WeakMap:Map, {
  init(){
    self.items = setmetatable({}, { __mode = "k" })
  }
}

function default_to(default)
  return function(self, value)
    return value == nil and default or value
  end
end


class! @into_collectible("collect") Sandbox, {
  init(){
    self.env = {}
  }
  set(key, val) {
    self.env[key] = val
    return self
  }
  eval(code, name){
    return exec_sandboxed(code, name or "lulu::sandbox", self.env)
  }
}


local function extract_serializable(o, parent)
  parent = parent or {}
  if type(o) ~= "table" then
    if type(o) == "string" or type(o) == "number" or type(o) == "boolean" then
      return o
    else
      return nil
    end
  end

  if parent[o] then
    return nil
  end
  parent[o] = true

  local result = {}
  for k, v in pairs(o) do
    if type(k) == "string" or type(k) == "number" then
      if type(k) == "number" or k:sub(1, 2) ~= "__" then
        local sv = extract_serializable(v, parent)
        if sv ~= nil then
          result[k] = sv
        end
      end
    end
  end

  return result
end

function Serializable(type)
  return function(_class)
    () _class:serialize =>
      return serde[type].encode(extract_serializable(self))
    end
    (arg) _class:deserialize =>
      return _class(serde[type].decode(arg))
    end
    return _class
  end
end
