
local f = Option.Some("SS")
local nums = vec! { 10 }

function lml_create()

end

cfg! OS, {
  linux {
    print("Hello")
  }
}

match! f, {
  Some {
    print("hello", ff)
  }
  _ {}
}

class! ff, {
  init(){
    self.name = ""
  }
}