import! utils, { "./utils.lua" }
import! smn, { "./src/something.lua" }
include_bytes! bbb, { "./main.txt" }

fprint(utils)
fprint(smn)
print(bbb)

test! {
  add {
    assert(smn == "somn", "smn should be somn")
  }
}