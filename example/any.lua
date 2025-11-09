using {
  lulib.console
}
console.enter_alternate_screen()

print(console.size())
print(console.pos())

console.poll_events(function(e)
  print(e)
end)

-- console.leave_alternate_screen()