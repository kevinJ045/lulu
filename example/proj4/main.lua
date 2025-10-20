import! utils, { "./utils.lua" }
import! smn, { "./src/something.lua" }
local bbb = include_bytes! { "./some.txt" }

fprint(utils)
fprint(smn)
print(bbb:to_string())

test! {
  add {
    assert(smn == "somn", "smn should be somn")
  }
}