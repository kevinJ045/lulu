using {
  lulib.clap
}

local cmd = clap.Command({
  name = "lulu",
  version = "1.0",
  about = "example"
})

cmd:arg("file", { default = ".", help = "file" })
  :flag("build", { short = "b", long = "build", help = "dd" })
  :arg("into", { short = "i", required = false })
  :arg("items*", { required = false, trailing = true })

local result = cmd:parse({ "dosmn", "-b", "-i sjjs", "fjfjf", "jfjfj", "djdjjd" })

fprint(result)