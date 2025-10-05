
hello! { "Someone" }

for_each! item, { range(1, 10) }, {
  print(item)
}

return {
  name = mod.conf.manifest.name
}


