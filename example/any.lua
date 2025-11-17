using {
  static 'count' (ArcMutex(0))
}

while count() < 10 do
  count(count + 1)
end

print(count())


