local items = {0, 5, 10}

macro {
  for_each ($key, $iterator, $expr) {
    for $key in ipairs($iterator) do
      $expr
    end
  }
}

for_each! item, items {
  print(item)
}