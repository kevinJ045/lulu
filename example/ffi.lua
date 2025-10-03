local ffi = require("ffi")

ffi.cdef[[
ffi_type(i32, ptr) void* webview_create(int, void*)
ffi_type(i32, ptr) void webview_run(void*)
ffi_type(i32, ptr) void webview_set_html(void*, const char*)
ffi_type(ptr) webview_destroy = -> ffi::void
ffi_type(ptr) webview_run = -> ffi::void
ffi_type(ptr) webview_terminate = -> ffi::void
ffi_type(ptr) webview_get_window = -> ptr
ffi_type(ptr, buf) webview_set_title = -> ffi::void
ffi_type(ptr, i32, i32, i32) webview_set_size = -> ffi::void
ffi_type(ptr, buf) webview_navigate = -> ffi::void
ffi_type(ptr, ptr) webview_set_html = -> ffi::void
ffi_type(ptr, buf) webview_init = -> ffi::void
ffi_type(ptr, buf) webview_eval = -> ffi::void
ffi_type(ptr, buf, unsafe, ptr) webview_bind = -> ffi::void
ffi_type(ptr, buf) webview_unbind = -> ffi::void
ffi_type(ptr, buf, i32, buf) webview_return = -> ffi::void
]]

local lib = ffi.load("/home/makano/workspace/rew.smn/demo.so")


print(lib.add(1, 1))
print(ffi.string(lib.ssmmtt("sjjsjs")))

-- Callback
local function cb(x)
  print("Lua callback got:", x)
  return x + 1
end

-- Wrap callback in ffi.cast
local c_callback = ffi.cast("int(*)(int)", cb)
print("dofn(cb) =", lib.dofn(c_callback))
c_callback:free() -- free when done!

-- Struct
local s = ffi.new("Structt", { dd = 99 })
local ret = lib.doStruct(s)
print("Returned struct dd =", ret.dd)