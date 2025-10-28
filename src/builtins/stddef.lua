-- Lulu standard definitions for static analysis (generated)

--=== Macros & special syntax ===--
---@class LuluDecorator
decorator = {} ---@type table<string, any>

---@class LuluClass
class = {} ---@type table<string, any>

---@param name string
---@param value any
---@return any
function enum(name, value) end

---@param str string
---@return string
function f(str) end -- Lulu’s string interpolation operator (f"…")

---@class Serde
---@field json { encode: fun(tbl:table):string, decode: fun(s:string):table }
---@field yaml { encode: fun(tbl:table):string, decode: fun(s:string):table }
serde = {}

---@param obj any
---@param parent? table
---@return table|nil
function extract_serializable(obj, parent) end

---@param _stype table
---@return fun(_self:any, value:any, name:string):any
function Deserializable(_stype) end

---@param _stype string
---@return fun(_class:table, name?:string):table
function Serializable(_stype) end


--=== Async/Future ===--

---@class Future
---@field co thread
---@field done boolean
---@field result any
---@field error any
---@field onError fun(err:any)
---@field onAfter fun(res:any):any
Future = {}

---@param fn fun()
---@return Future
function Future.new(fn) end

function Future:poll(...) end
function Future:last() end
function Future:await() end
---@param cb fun(any):any
---@return Future
function Future:after(cb) end
---@param cb fun(any):any
---@return Future
function Future:catch(cb) end

---@param fn fun()
---@return Future
function async(fn) end



---@class Vec<T>
---@field items T[]
Vec = {}

function Vec:push(...) end
function Vec:pop() end
function Vec:len() end
---@param index integer
---@return Vec
function Vec:get(index) end
---@param index integer
---@param value T
---@return Vec
function Vec:set(index, value) end

---@param fn fun(v:T, i:integer, self:Vec<T>):any
---@return Vec<T>
function Vec:map(fn) end
---@param fn fun(v:T, i:integer, self:Vec<T>):boolean
---@return Vec<T>
function Vec:filter(fn) 
---@param sep string
---@return string
function Vec:join(sep) end
---@return Vec<T>
function Vec:clone() end
---@param ... Vec | table
---@return Vec<T>
function Vec:extend(...) end


---@class Set<T>
---@field items table<T, boolean>
Set = {}

---@return Set
function Set:add(value) end
---@return Set
function Set:remove(value) end
---@return boolean
function Set:has(value) end
function Set:values() end
---@return Set
function Set:clone() end

---@class WeakSet:Set
WeakSet = {}

---@class Map<K,V>
---@field items table<K,V>
Map = {}

---@return Map
function Map:set(key, value) end
---@return V
function Map:get(key, default) end
---@return boolean
function Map:has(key) end
---@return Map
function Map:remove(key) end
---@return Vec
function Map:keys() end
---@return Vec
function Map:values() end
---@return Map
function Map:clone() end

---@class WeakMap:Map
WeakMap = {}

---@class String
---@field str string
String = {}

---@param s string
---@return String
function String:push_str(s) end
---@param s any
---@return String
function String:push_string(s) end
---@param sep string
---@return Vec
function String:split(sep) end
---@param prefix string
---@return boolean
function String:starts_with(prefix) end
---@param prefix string
---@return boolean
function String:ends_with(suffix) end
---@param pattern string
---@param repl string | fun(...:string):string
---@return String
function String:replace(pattern, repl) end
---@return String
function String:clone() end
---@return string
function String:__tostring() end


--=== Enums ===--

---@param name string
---@return table
function make_enum(name) end

---@param enum_table table
---@param vname string
---@param names any
---@return table<string, any>
function make_enum_var(enum_table, vname, names) return {} end

---@param enum_table table
---@param vname string
---@param names any
---@return fun(...):table<string, any>
function make_enum_var_dyn(enum_table, vname, names) return function() end end 

function get_enum_var_name(var) end

---@class table
---@field Some fun(...):table
---@field None table
Option = {}

---@class table
---@field Ok fun(...):table
---@field Err table
Result = make_enum("Result")

Some = Option.Some
None = Option.None
Ok = Result.Ok
Err = Result.Err


--=== Classes & Utilities ===--

---@param class_raw table
---@param parent any
---@return table
function make_class(class_raw, parent) end
function instanceof(obj, class) end
function empty_class() end
function namespace(tbl) end


---@param _name string
---@return fun(tbl:any)
function into_collectible(_name) end

---@param ... any
---@return function
function validate_type(...) end

---@param a any
---@param b any
---@return boolean
function iseq(a, b) end

function index_of(object, item) end

---@return fun(any, value):any
function default_to(default) end

---@return fun(any, value):any
function default_not_nil(self, value, name) end

---@class Sandbox
---@field env table
Sandbox = {}
function Sandbox:set(key, val) end
function Sandbox:eval(code, name) end



--=== Lulib ===--

---@class Lulib
lulib = {}

---@param env? table
---@param name string
---@return any
function lulib.__call(env, name) end


---@class ByteArray
ByteArray = {}

---@return string
function ByteArray:to_string() end

---@return Vec
function ByteArray:to_vec() end

---@class Net
net = {}

---@class Response
---@field status number
---@field body ByteArray
---@field uri string
---@field headers table<string, string>
local Response = {}

---@class Request
---@field status number
---@field body any
---@field uri string
---@field headers table<string, string>
local Request = {}

---@class Http
---@field request fun(table):Response
---@field get fun(table):Response
---@field post fun(table):Response
---@field put fun(table):Response
---@field patch fun(table):Response
---@field delete fun(table):Response
---@field serve fun(f: fun(request: Request):any)
net.http = {}

---@class Archive
archive = {}

---@class KvDb
kvdb = {}

---@class Threads
threads = {}

---@param ... any
---@return function
function using(...) end

---@param ... any
---@return nil
function fprint(...) end

---@param ... any
---@return function
function lookup_dylib(...) end

---@param ... any
---@return any
function require_cached(...) end

---@type string
current_path = ""

---@class Mod
---@field name string
mod = {}