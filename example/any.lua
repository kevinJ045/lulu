using {
  static 'count' (ArcMutex(0))
}

(ctx, data) @Usage myUsage =>
  print(data.mod.something)
  data.mod.something = "hi"
end

using(myUsage)
using(myUsage)

while count() < 10 do
  count(count + 1)
end

print(count())


