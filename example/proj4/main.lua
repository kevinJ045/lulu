import! utils, { "./utils.lua" }
import! smn, { "./src/something.lua" }

test! {
  add {
    assert(smn == "somn", "smn should be somn")
  }
}