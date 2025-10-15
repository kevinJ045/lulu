class! @register() Example:Base(arg1, arg2), {
  @log()
  method(@validated x, y) {
    print(x, y)
  }

  normal(x, y, z){
    print(x, y, z)
  }
}
