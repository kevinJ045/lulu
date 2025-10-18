
local Something = {
  value = "sds",
  valuef = function(a) return a end
}

local value = "ss"

match! value, {
  not Something.value or Something.valuef("fff") {
    print("not sds or fff")
  }
  (val == "ss") {
    print("ss")
  }
  "sss" or "ss" or "s" {
    print("matched ss")
    -- can have multiple statements chained with or
  }
  "dd" {
    -- this exists
  }
  _ {}
}