enum! Option, {
  Some(value),
  None
}

local myval = Option.None

match! myval, {
  Option.None {
    print("has no value")
  }
}


