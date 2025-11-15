using {
  lulib.console
}

local s = cs"hi"
  .rgb({100, 255, 0})
  .underline({0, 0, 100})
  .bold()

console.print(
  s(),
  console.red("hi"),
  console.italic("sss")
)
