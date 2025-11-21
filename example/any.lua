
local {
  Some(content),
  None
} -< Option



match(Option.Some('somestuff')) do
  in Option.Some then
    print('Some')
    print(val.content)
  in _ then
    print("nope")
end

match(x) do
  in (val > 5) then
    return "hi"
end