
local myrec = rec {
  deterministic = 'a',
  a = 1,
  b = 4,
  f = 5
}

print(Vec(#myrec))
print(Vec(-myrec))

myrec.ff = "ff"

for k, v in myrec() do
  print(k, v)
end

print(myrec)
