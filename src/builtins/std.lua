
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

function namespace(tbl, chunk)
  chunk = chunk or function() end
  setfenv(chunk, setmetatable(tbl or {}, { __index = _G }))
  return chunk(tbl)
end

function make_class(class_raw, parent)
  local class = class_raw
  class.__index = class
  local inits = {}

  function class:__call_init(...)
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

  if first and type(first) == "table" and first.__is and first.__is(first, second) then
    return true
  end
  
  if instanceof(first, second) then
    return true
  end

  return result
end

function instanceof(obj, class)
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
  Some(content)
  None
}

Some = Option.Some
None = Option.None

Option.func.unwrap = function(item)
  return item.content and item.content or nil
end

enum! Result, {
  Ok(content)
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
    print(value)
    return value or default
  end
end