using {
  static 'count' (ArcMutex(0)),
  
  namespace({ name = "dd" }, { dd = "name" }),
}

while count() < 10 do
  count(count + 1)
end

print(count())
