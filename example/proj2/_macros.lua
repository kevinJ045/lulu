macro {
  for_each ($key, $iterator, $expr) {
    for $key in ipairs($iterator) do
      $expr
    end
  }
}
macro {
  hello ($something) {
    print("Hello, " .. $something)
  }
}