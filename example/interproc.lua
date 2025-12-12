
using {
  lulib.interproc,
  lulib.serde
}

messager = InterprocSocket("mysoc"):into()
  .with(interproc.JSON)
  .listen(function(message) fprint(message) end)

messager.send({
  name = "ff"
})

messager.stop()
