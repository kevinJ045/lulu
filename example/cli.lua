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
            error(types[i] and f"Expected {t} for function at argument {i}. Found {type(arg)}" or (
              #types > #args and f"Expected {#types} arguments for function, given {#args}" or f"Extra args for function."
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
    }
    (_self, value) {
      return verify(value)[1]
    }
  }
end

local is_cool = decorator! {
  (_enum, variant) {
    dynamic {
      return function(...)
        local r = variant(...)
        r.coolness = "THE VARIANT IS COOL"
        return r
      end
    }
    static {
      variant.coolness = "THE VARIANT IS COOL"
      return variant
    }
  }
}

enum! Token, {
  @validate_type('string')
  @is_cool
  String(name),
  
  @is_cool
  EOF
}

class! Something(@validate_type(Token.String) token), {
  @validate_type('string', 'number')
  dosmn(name, id){
    print(name, id)
  }
}

local s = Token.String("ssks")
local s2 = Something(s)
s2:dosmn("ssshs", 4)
fprint(s)
print(s2.token)
