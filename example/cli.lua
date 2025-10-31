local proc = spawn("ping -s 5 google.com")

-- write to stdin
proc:write("foo\nbar\nfoo bar\n")

proc:close_stdin()

-- read lines
while true do
  local line = proc:read()
  if line then
    print("stdout:", line)
  end
  local status = proc:wait_nonblocking()
  if status then break end
end

