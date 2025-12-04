
macro {
  vec ($block) {Vec({ $block }):into()}
}

macro {
  as_vec ($block) {Vec($block)}
}

function table.keys(t)
  local ks = {}
  for k in pairs(t) do
    table.insert(ks, k)
  end
  return ks
end

function table.values(t)
  local vs = {}
  for _, v in pairs(t) do
    table.insert(vs, v)
  end
  return vs
end

function table.entities(t)
  local e = {}
  for k, v in pairs(t) do
    table.insert(e, { k, v })
  end
  return e
end

function table.from_entities(t)
  local o = {}
  for _, v in ipairs(t) do
    o[v[1]] = v[2]
  end
  return o
end

function dump_item_into_string(o, indent)
  indent = indent or 0
  if type(o) == 'table' then
    local s = ''
    if o.__class then
      s = "Instance "
    end
    s = s .. '{\n'
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

function ns_inherit_from(...)
  local parents = { ... }

  return setmetatable({}, {
    __index = function(_, key)
      if key == "__gns" then
        return true
      end

      for _, parent in ipairs(parents) do
        if parent ~= nil then
          local value = parent[key]
          if value ~= nil then
            return value
          end
        end
      end

      return rawget(_G, key)
    end
  })
end

local function mkproxy(parent)
  local proxy = {}

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
    end
  })

  return proxy
end

function namespace(tbl, ...)
  local namespaces = {...}
  local ns = tbl

  if #namespaces > 0 then
    ns = ns_inherit_from(tbl, ...)
  end

  return function(chunk)
    local t = ns
    if ns ~= nil and ns.__gns then
      t = ns
    else
      t = setmetatable(t, { __index = _G })
    end
    if type(chunk) == "table" then
      setmetatable(chunk, { __index = t })
      return
    end
    chunk = chunk or function() end
    setfenv(chunk, t)
    local r = chunk(ns) or ns
    r.__static = mkproxy(r)
    return r
  end
end

function make_enum(name)
  local e = {}

  function e.is(obj, variant)
    if type(obj) ~= 'table' then return false end
    if obj.__enum == nil then return false end
    if variant then
      if obj.__enum_var == nil then return false end
      return obj.__enum_var == variant or obj.__enum_var == variant.__enum_var
    end
    return obj.__enum == e
  end

  local _create_funcs = {}

  function e.on_create(fn)
    table.insert(_create_funcs, fn)
    return e
  end

  e.__is_enum = true
  e.__name = name or ""
  e.__static = mkproxy(e)

  return setmetatable(e, {
    __newindex = function(tbl, k, v)
      if type(v) == "function" or (type(v) == "table" and v.__enum_var) then
        for _, v in ipairs(_create_funcs) do
          v(v, k)
        end
      end

      rawset(tbl, k, v)
    end
  })
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
  return -1
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
      local item = enum_table.__static[key]
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

  class.__static = mkproxy(class)

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

function empty_class(self)
  if self then
    if type(self) == "table" and self.__class then
      return self
    end
  end
  return {
    __class = {
      empty = true
    }
  }
end

function instanceof(obj, class)
  if not obj then return false end
  if not class and obj then return false end
  if type(obj) ~= "table" then return false end
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

Option::unwrap = function(item)
  return item.content and item.content or nil
end

Option::is_some = function(item)
  return item.content and true or false
end

enum! Result, {
  Ok(content),
  Err(err)
}

Ok = Result.Ok
Err = Result.Err

Option::is_ok = function(item)
  return item.content and true or false
end

Result::unwrap = function(item)
  return item.content and item.content or item.err
end



local function handle_trait_value(self, key, val, def)
  local v = val or def

  if type(v) == "table" and type(v[1]) == "function" then
    v = derive.with(unpack(v))
  end

  if type(v) == "table" and v.__is_decorated then
    local d = def
    if v == d or d == nil then
      d = nil
    else
      d = handle_trait_value(self, key, def)
    end
    return v.__func(self, key, d)
  elseif def != nil then
    return def
  end

  return v
end

local function apply_traits(new, traits, options, args, on_function)
  for _, trait in ipairs(traits) do
    for k, v in pairs(trait) do
      if k != "__is_trait" or k != "__init" or k != "__on_apply" or k != "__apply" then
        if type(v) != "function" then
          new[k] = handle_trait_value(new, k, v, new[k] or options[k])
        else
          if on_function then
            on_function(new, k, v)
          else
            new[k] = function(...)
              return v(new, ...)
            end
          end
        end
      end
    end
    trait.__init(new, options, on_function, unpack(args))
  end
end

function trait(template, ...)
  local traits = {...}

  template.__is_trait = true

  return function(func)
    template.__init = function(self, options, on_function, ...)
      if #traits > 0 then
        apply_traits(self, traits, options, {...}, on_function)
      end
      func(self, ...)
    end
    template.__apply = function(into, options, args, on_function)
      apply_traits(into, {template}, options, args, on_function)
    end
    return template
  end
end

function with_trait(...)
  local traits = {...}
  return function(_class)
    function _class:init(...)
      apply_traits(self, traits, self, { ... }, function(_, k, v)
        if not _class[k] then
          _class[k] = function(self, ...)
            return v(self, ...)
          end
        end
      end)
    end

    for _, trait in ipairs(traits) do
      if trait.__on_apply then
        trait.__on_apply(_class)
      end
    end

    return _class
  end
end

derive = setmetatable({}, {
  __call = function(tbl, template, ...)
    local traits = {...}
    return function(func)
      local p = setmetatable({
        __static = {
          __template = template,
          __traits = traits
        }
      }, {
        __call = function(tbl, options, ...)
          local new = { __class = tbl }

          if not options then options = {} end

          apply_traits(new, traits, options, { ... })

          for k, v in pairs(template) do
            new[k] = handle_trait_value(new, k, template[k], options[k])
          end

          for k, v in pairs(tbl.__static) do
            new[k] = function(...)
              v(new, ...)
            end
          end

          func(new, ...)
          return new
        end,
        __newindex = function(tbl, k, v)
          if type(v) == "function" then
            tbl.__static[k] = v
          end

          return rawset(tbl, k, v)
        end
      })

      for _, trait in ipairs(p.__static.__traits) do
        if trait.__on_apply then
          trait.__on_apply(p)
        end
      end

      return p
    end
  end
})

derive.with = function(...)
  local decos = {...}
  return {
    __is_decorated = true,
    __func = function(instance, name, def)
      local default = def
      for _, deco in ipairs(decos) do
        default = deco(instance, default, name)
      end
      return default
    end
  }
end


derive.satiates = function(thing, ...)
  local traits = {...}
  local satiates = true

  for _, trait in ipairs(traits) do
    local satiated_all = true

    if thing.__class then
      for k in pairs(trait) do
        if k:sub(1, 2) != '__' then
          if not thing[k] then
            satiated_all = false
            thing = thing.__class
            break
          end
        end
      end
    end

    if not satiated_all and thing.__static and thing.__static.__traits then
      if index_of(thing.__static.__traits, trait) < 0 then
        satiated_all = false
      end
    end

    satiates = satiates and satiated_all
    if not satiates then break end
  end

  return satiates
end

function enum_from_string(enum)
  if not enum::from then enum::from = function(idx)
    for k, v in pairs(enum) do
      if type(v) == "table" and (v.index == idx or string.lower(k) == idx or k == idx) then
        return v
      end
    end
  end end

  return enum
end

local function into_indexed_enum(enum)
  if not enum::index then enum::index = function(idx)
    for k, v in pairs(enum) do
      if type(v) == "table" and (v.index == idx or string.lower(k) == idx or k == idx) then
        return v.index
      end
    end
  end end

  enum_from_string(enum)
end

function enum_index(idx)
  return function(enum, variant)
    into_indexed_enum(enum)

    variant.index = idx

    return variant
  end
end

function enum_indexed(idx)
  return function(enum)
    into_indexed_enum(enum)

    local index = idx

    for k, v in pairs(enum) do
      if type(v) == "table" and v.__enum_var then
        if v.index == nil then
          v.index = index
          index += 1
        end
      end
    end

    return enum
  end
end

function into_collectible(name, indexible)
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

          if indexible then
            if parent[indexible][key] then
              val = parent[indexible][key]
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
        for i, t in ipairs(types) do
          local arg = args[i]
          if t == '!' then

          elseif type(arg) != t and not iseq(arg, t) then
            if t != "number" and t != "string" then
              t = f"abstract({tostring(t)})"
            end
            error(t and f"Expected {t} for {name} at argument {i}. Found {type(arg)}" or (
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
    (_func) {
      return function(...)
        return method(unpack(verify(...)))
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

function map_into(fn)
  return function(_self, value, name)
    if type(fn) == "function" then
      return fn(value, _self, name)
    else
      return value or fn
    end
  end
end

class! @into_collectible("collect", "items") Vec, {
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

  keys() {
    return Vec(table.keys(self.items))
  }

  values() {
    return Vec(table.values(self.items))
  }

  __tostring(){
    try_catch! {
      return "[" .. table.concat(self.items, ", ") .. "]"
    }, {
      return "[Unstringable Table (" .. #self.items .. ")]"
    }
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

  serialize(keep){
    local mapped = self:map(function(item)
      if item.serialize then
        return item:serialize()
      else
        return "\"" .. tostring(item) .. "\""
      end
    end)

    if keep then return mapped end

    return mapped:__tostring()
  }

  deserialize(thing, _type){
    local items = Vec()
    for k, v in pairs(thing) do
      if type(_type) == "table" and _type.__call_init then
        items:push(_type(v))
      else
        items:push(v)
      end
    end
    return items
  }

  of(_type){
    return {
      __is_vec = true,
      deserialize = function(thing, value)
        if not value then return value end
        return Vec:deserialize(value, _type)
      end
    }
  }
}

function extract_serializable(o, parent)
  parent = parent or {}
  if type(o) ~= "table" then
    if type(o) == "string" or type(o) == "number" or type(o) == "boolean" then
      return o
    else
      return nil
    end
  end

  if instanceof(o, Vec) then
    o = o.items
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

function Deserializable(_stype)
  return function(_self, value, name)
    if not value then return value end
    local deserialize = not instanceof(value, _stype)
    if _stype.__is_vec then
      deserialize = not instanceof(value, Vec)
    end
    if deserialize then
      return _stype:deserialize(value)
    else
      return value
    end
  end
end

function Serializable(_stype)
  return setmetatable(trait({
    serialize = function(self)
      return serde[_stype].encode(extract_serializable(self))
    end,
    __on_apply = function(_class)
      local s = _class.__static
      if _class.__call_init then
        s = _class
      end
      s.deserialize = function(arg)
        if type(arg) == "string" then
          arg = serde[_stype].decode(arg)
        end
        return _class(arg)
      end
    end
  })(function(self) end), {
    __call = function(tbl, _class)
      () _class:serialize =>
        return serde[_stype].encode(extract_serializable(self))
      end
      () _class:__tostring =>
        return self:serialize()
      end
      (arg) _class:deserialize =>
        if type(arg) == "string" then
          arg = serde[_stype].decode(arg)
        end
        return _class(arg)
      end

      return _class
    end
  })
end

Clone = trait({
  clone = function(self)
    return self.__class(self)
  end
})(function(self) end)

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

  upper(){
    self.str = string.upper(self.str)
    return self
  }

  lower(){
    self.str = string.lower(self.str)
    return self
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

function default_not_nil(self, value, name)
  if value == nil then
    error(f"Param {name} should not be nil.")
  end
  return value
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


lulib = {}
setmetatable(lulib, {
  __call = function(tbl, name, env)
    return function()
      return request_env_load(env, name)
    end
  end,
  __index = function(tbl, key)
    if key == "from" then
      return function(a)
        return function()
          return require_cached(a, false)
        end
      end
    end
    return function()
      local tbl = request_env_load(key)
      if type(tbl) == "table" and tbl.__include then
        for _, k in ipairs(tbl.__include) do
          request_env_load(k)
        end
      end
    end
  end
})

function dylib(dylib)
  return function(name)
    local function load(ctx)
      ctx[name] = ffi.load(ctx.lookup_dylib(dylib))
    end
    return function(ctx)
      if type(ctx) == "string" then
        try_catch! {
          ffi.cdef(ctx)
        }, {}
        return load
      else
        return load(ctx)
      end
    end
  end
end

function dylib_cdef(def)
  return function()
    ffi.cdef(def)
  end
end

function into_global(key, value)
  _G[key] = value
  return value
end

function globalize(thing, name)
  return into_global(name, thing)
end

local _keystore = {}

function static(key, val)
  local function _init(ctx)
    if _keystore[f"{ctx.mod.name}::{key}"] then
      ctx[key] = _keystore[f"{ctx.mod.name}::{key}"]
      return ctx[key]
    end
    ctx[key] = val
    _keystore[f"{ctx.mod.name}::{key}"] = ctx[key]
    return ctx[key]
  end
  if val then return _init end
  return function(v)
    val = v
    return _init
  end
end

function keystore(ctx)
  ctx.kget = function(key)
    return _keystore[f"{ctx.mod.name}::{key}"]
  end
  ctx.kset = function(key, val)
    _keystore[f"{ctx.mod.name}::{key}"] = val
  end
end

runtime = {}

runtime.once = function(usage)
  return function(ctx)
    if _keystore[f"ran-{ctx.mod.name}"] then
      return nil
    end
    _keystore[f"ran-{ctx.mod.name}"] = true
    return usage(ctx, 'once')
  end
end

local usage_data = {}
local usage_data_per_mod = {}
function Usage(func)
  return function(ctx, ...)
    if not usage_data_per_mod[ctx.mod.name] then
      usage_data_per_mod[ctx.mod.name] = {}
    end
    return func(ctx, { global = usage_data, mod = usage_data_per_mod[ctx.mod.name] }, ...)
  end
end
