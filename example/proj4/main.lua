import! utils, { "./utils.lua" }
import! smn, { "./src/something.lua" }
include_bytes! bbb, { "./main.txt" }

test! {
  add {
    assert(smn == "somn", "smn should be somn")
  }
}