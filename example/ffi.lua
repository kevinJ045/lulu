local ffi = require("ffi")

ffi.cdef[[
typedef struct Structt {
  int dd;
} Strctt;

int add(int a, int b);
const char* ssmmtt(const char* a);
int dofn(int (*cb)(int));
Strctt doStruct(Strctt d);
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
local s = ffi.new("Strctt", { dd = 274657 })
local ret = lib.doStruct(s)
print("Returned struct dd =", ret.dd)