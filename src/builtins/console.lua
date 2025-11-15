
cs = function(string)
  local parent = console.string(string)
  local proxy = {}

  setmetatable(proxy, {
    __call = function()
      return parent
    end,
    __index = function(_, key)
      local val = parent[key]

      if type(val) == "function" then
        return function(...)
          local result = val(parent, ...)
          if result == parent then
            return proxy
          elseif not result then
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