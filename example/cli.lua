
-- class! @getter("getter", {

-- }) MyClass, {}

local a = "ss"
local b = 3
local c = vec!{1, 2, 3}
local d = function()
  
end

local f = {
  k = "ss"
}

local g = {1,2,3}

fprint(collect! { a, b, c, d, e = "the e", ..f, ...g })