local items = { "one", "two", "three" }

local list = lml! {
  <list>
    {
      Vec(items):map(function(item)
        return <item text={item} />
      end).items
    }
  </list>
}