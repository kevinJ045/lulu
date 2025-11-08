

local ns = {
  x = 1
}

local ns2 = {
  y = 2
}

in local my_namespace and ns and ns2

  y += 10

  something = in do
    -- some code block,
    -- whatever is returned here is the actual value
    return in if x > 0 then
      return "hi"
    end
  end

end

print(my_namespace.y)
print(my_namespace.something)