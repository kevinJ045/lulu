
local f = Option.Some("SS")
local nums = vec! { 10 }

function lml_create()

end

lml! {
  <example />
}

cfg! OS, {
  linux {
    print("Hello")
  }
}

match! f, {
  Some {
    print("hello")
  }
  _ {}
}

class! ff, {
  init(){
    self.name = ""
  }
}