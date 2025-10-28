---@meta
-- Lulu standard definitions for static analysis (generated)

--=== Macros & special syntax ===--

---@class LuluMacro
macro = {} ---@type table<string, any>

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
---@return fun(_class:table):table
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
function Future:after(cb:fun(any):any):Future end
function Future:catch(cb:fun(any):any):Future end

---@param fn fun()
---@return Future
function async(fn) end



---@class Vec<T>
---@field items T[]
local Vec = {}

function Vec:push(...) end
function Vec:pop() end
function Vec:len() end
function Vec:get(index:integer):any end
function Vec:set(index:integer, value:any) end
function Vec:map(fn:fun(v:any, i:integer, self:Vec):any):Vec end
function Vec:filter(fn:fun(v:any, i:integer, self:Vec):boolean):Vec end
function Vec:join(sep:string):string end
function Vec:clone():Vec end
function Vec:serialize(keep:boolean):string|Vec end
function Vec:deserialize(thing:any, _type:any):Vec end
function Vec:of(_type:any):table end
function Vec:extend(...) end

---@class Set<T>
---@field items table<T, boolean>
local Set = {}

function Set:add(value:any):Set end
function Set:remove(value:any):Set end
function Set:has(value:any):boolean end
function Set:values():Vec end
function Set:clone():Set end

---@class WeakSet:Set
local WeakSet = {}

---@class Map<K,V>
---@field items table<K,V>
local Map = {}

function Map:set(key:any, value:any):Map end
function Map:get(key:any, default:any):V end
function Map:has(key:any):boolean end
function Map:remove(key:any):Map end
function Map:keys():Vec end
function Map:values():Vec end
function Map:clone():Map end

---@class WeakMap:Map
local WeakMap = {}

---@class String
---@field str string
local String = {}

function String:push_str(s:string):String end
function String:push_string(s:any):String end
function String:split(sep:string):Vec end
function String:starts_with(prefix:string):boolean end
function String:ends_with(suffix:string):boolean end
function String:replace(pattern:string, repl:string):String end
function String:clone():String end
function String:__tostring():string end


--=== Enums ===--

---@class Enum
---@field func table<string, function>
local Enum = {}

function make_enum(name:string):Enum end
function make_enum_var(enum_table:any, vname:string, names:any, ...:any) end
function make_enum_var_dyn(enum_table:any, vname:string, names:any):function end
function get_enum_var_name(var:any):string end

Option = make_enum("Option")
Result = make_enum("Result")

Some = Option.Some
None = Option.None
Ok = Result.Ok
Err = Result.Err


--=== Classes & Utilities ===--

function make_class(class_raw:any, parent:any):any end
function instanceof(obj:any, class:any):boolean end
function empty_class():any end
function namespace(tbl:any):function end


---@param _name string
---@return fun(tbl:any):any
function into_collectible(_name) end

---@param ... any
---@return function
function validate_type(...) end

function iseq(a:any, b:any):boolean end
function index_of(object:any, item:any):integer|nil end

function default_to(default:any):fun(self:any, value:any):any end
function default_not_nil(self:any, value:any, name:string):any end

---@class Sandbox
---@field env table
Sandbox = {}
function Sandbox:set(key:any, val:any):Sandbox end
function Sandbox:eval(code:string, name?:string):any end



--=== Lulib ===--

---@class Lulib
lulib = {}

---@param env? table
---@param name string
---@return any
function lulib.__call(env, name) end



---@class Net
---@field http Http
---@class tcp Tcp
---@class udp UDP
net = {}

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
---@return function
function lookup_dylib(...) end

---@param ... any
---@return function
function lookup_dylib(...) end

---@type string
current_path = ""

---@class Mod
---@field name string
mod = {}