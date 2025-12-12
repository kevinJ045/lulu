# Compression

A simple `zstd` compression binding for lulu.

## Methods

- **`compression.compress(data: ByteArray, size)`**: Returns a `ByteArray` of the compressed data.
- **`compression.decompress(data: ByteArray)`**: Returns a `ByteArray` of the decompressed data.

```lua
using { lulib.compression }

local content = read("./path/to/some-file")
-- Or just
local content = ByteArray("Some long content")

local compressed = compression.compress(content)
local decompressed = compression.decompress(compressed)

print(content:len())
print(compressed:len())
print(decompressed:len())

```
