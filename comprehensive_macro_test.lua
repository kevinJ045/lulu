-- Test different macro patterns
local items = {1, 2, 3}

-- Simple macro
macro {
  greet ($name) {
    print("Hello, " .. $name)
  }
}

-- For-each macro like the example
macro {
  for_each ($key, $iterator, $expr) {
    for $key in ipairs($iterator) do
      $expr
    end
  }
}

-- Conditional macro
macro {
  when ($condition, $then_block) {
    if $condition then
      $then_block
    end
  }
}

-- Usage
greet! "Lulu"

for_each! item, items {
  print("Item:", item)
}

local x = 10
when! x > 5 {
  print("x is big enough")
}