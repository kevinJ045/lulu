
local function multivalue(Class, vec)
  vec = vec:map(function(v)
    return Class(v)
  end)
  function vec:select(...)
    local args = { ... }
    return self:map(function(v)
      return v:select(unpack(args))
    end)
  end
  return vec
end

function kvdb.table_of(db, name, Class)

  local tabl = db:table(name, Class.__keys)
  Class.__db = db
  Class.__db_table = tabl

  local tbl = {}

  tbl.real = tabl;

  function tbl:find(name, value, limit, offset)
    return multivalue(Class, Vec(tabl:find(name, value, limit, offset)))
  end

  function tbl:matches(name, value, limit, offset)
    return multivalue(Class, Vec(tabl:matches(name, value, limit, offset)))
  end

  function tbl:gt(name, value, limit, offset)
    return multivalue(Class, Vec(tabl:gt(name, value, limit, offset)))
  end
  function tbl:lt(name, value, limit, offset)
    return multivalue(Class, Vec(tabl:lt(name, value, limit, offset)))
  end

  function tbl:all(limit, offset)
    return multivalue(Class, Vec(tabl:find(limit, offset)))
  end

  function tbl:update(id, value)
    return tabl:update(id, extract_serializable(value))
  end

  function tbl:remove(id, value)
    return tabl:remove(id, value)
  end

  function tbl:insert(value)
    return tabl:insert(extract_serializable(value))
  end

  function tbl:index(value)
    local v = tabl:index(value)
    if v then
      return Class(v)
    end
    return v
  end

  function tbl:all()
    return tabl:all()
  end

  function tbl:matches(field, pattern)
    return tabl:matches(field, pattern)
  end

  function Class:into()
    self.id = tbl:insert(self)
    return self
  end

  function Class:select(...)
    local args = { ... }
    if #args == 0 then
      return self
    end
    if #args == 1 and args[1]:gmatch(',') then
      args = String(args[1]):split(',').items
    end

    local keep = {}
    
    for _, key in ipairs(args) do
      keep[key] = true
    end

    for k, _ in pairs(extract_serializable(self)) do
      if not keep[k] then
        self[k] = nil
      end
    end

    return self
  end


  return tbl
end

function DBTable(...)
  local keys = {...}
  return function(class)
    class.__keys = keys
    class.__id = 1
    return class
  end
end

function kvdb.PrimaryId(self)
  return self.__db:id()
end

function to_db_id(db, id)
  return f"{db}:{id}"
end