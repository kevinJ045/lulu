
using {
  lulib.compression
}


local something = read("Cargo.toml")

local compressed = compression.compress(something)
local decompressed = compression.decompress(compressed)

print(something:len())
print(compressed:len())
print(decompressed:len())
