test! {
  syntax {
    import! _, { "./syntax.lua" }
  }
  classes {
    import! _, { "./classes.lua" }
  }
  dyn {
    import! _, { "./dyn.lua" }
  }
  single {
    import! _, { "./single.lua" }
  }
  async {
    try_catch! {
      import! _, { "./async.lua" }
    }, {}
  }
}