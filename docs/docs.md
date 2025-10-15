# Lulu Documentation

A simple Lua runtime that also helps with bundling Lua code, resolving Lua requirements, and building a final executable with libraries.

## Getting Started
you should be able to get the installation files [here](https://github.com/kevinJ045/lulu/releases/latest);

### Installation

**Linux:**
On most linux distros, this should work:
```bash
curl -fsSL https://raw.githubusercontent.com/kevinj045/lulu/main/install-linux.sh | bash
```
If it does not, you can download an archive from [here](https://github.com/kevinJ045/lulu/releases/latest) and add the `lulu` executable to your `PATH`.

**Windows:**
You can either run:
```bash
irm https://raw.githubusercontent.com/kevinj045/lulu/main/install-windows.ps1 | iex
```
Or download the installer exe from [here](https://github.com/kevinJ045/lulu/releases/latest).

## Basic Usage
You can run lua files as:
```bash
lulu run main.lua -- arg1 arg2
```

## Projects
A basic project has to have a `lulu.conf.lua` file in the root. Here's a simple structure.
```
your-project/
├── lulu.conf.lua
├── main.lua
└── .lib/
    ├── lulib/            # Copied .lulib files
    │   ├── package1.lulib
    │   └── package2.lulib
    └── dylib/            # Copied dynamic libraries
        ├── package1.so   # Linux
        ├── package1.dll  # Windows
        └── package1.dylib # macOS
```

```lua
--- lulu.conf.lua
--- Here's a basic structure for the `lulu.conf.lua`.
manifest = {
  name = "project_name",
  version = "1.0.0",
  tags = { "some", "tag" }
}

mods = {
  main = "main.lua"
}
```

### Creating new projects
You can create a new project with the command below:
```bash
lulu new -gi project-name
# -g, --git = github
# -i, --ignore = ignore prompts
# -t, --lib = initiate a library 
```

## The lulu config
The lulu config or `lulu.conf.lua` is the entry for your project, it tells lulu everything about your project. For example, how to build it, what dependencies it has, and even what files to select to bundle.

### Manifest
The `manifest` field is required, and should hold at least the name of the project.
**Example**: 
```lua
--- keep
manifest = {
  name = "project_name",
  version = "1.0.0",
  tags = { "some", "tag" }
  description = "A long description"
}
```

As of now, other than the name, the rest of the attributes remain unused inside of lulu, however, I have a plan to make them useful later on.

### Modules
The `mods` field is also another very required field. It tells lulu which files to add to the lua bytecode bundle.
**Example**: 
```lua
--- keep
mods = {
  main = "main.lua",
  utils = "src/utils.lua"
}
```

#### Modules names.
Names like `main` and `init` are required, `main` is required as the only entry to run or kickstart the bundle *therefore the can only be one `main`*, however, `init` is a namcepaced entry module into libraries. 

- ⚠️ **Note**: You cannot have duplicate names for modules in the same project.

### Fetch field
The `fetch` field tells lulu, when loaded from github, that this github repository is a library and it has something to fetch.

**Example**: 
```lua
--- keep
-- this will fetch the code and build the project from cache
fetch = "code"
-- this will download a `.lulib` from the internet and store 
-- it in the `.lib` folder in the current project
fetch = {
  lulib = "https://github.com/user/repo/releases/download/v1.0.0/package.lulib",
  -- Optional dynamic libraries
  include = {
    linux = {"https://github.com/user/repo/releases/download/v1.0.0/package-linux.so"},
    windows = {"https://github.com/user/repo/releases/download/v1.0.0/package-windows.dll"},
    macos = {"https://github.com/user/repo/releases/download/v1.0.0/package-macos.dylib"}
  }
}
```


### Dependencies
The `dependencies` field basically tells lulu to download and/or build these urls and include it to the current project.

**Example**: 
```lua
--- keep
dependencies = {
  -- From github
  "github:username/repo",
  "github:username/repo@branch",
  "github:username/repo@branch#commit",
  -- or a URL
  "https://github.com/user/repo/releases/download/v1.0.0/package.lulib"
}
```

#### For GitHub Repositories (`github:user/repo`)
1. **Check Fetch Field**: First downloads the `lulu.conf.lua` from the GitHub repository
2. **Handle Based on Fetch Field**:
   - If `fetch = "code"`: Clones the repository to cache
   - If `fetch = { lulib = "url", include = {...} }`: Downloads the `.lulib` file and platform-specific libraries
   - If no fetch field: Ignores the operation
3. **Build**: Executes the package's build process in the cache directory (only for cloned repositories)
4. **Copy**: Copies built artifacts to your project:
   - `.lulib` files go to `.lib/lulib/`
   - Dynamic libraries go to `.lib/dylib/` (platform-specific: `.so`, `.dll`, `.dylib`)

#### For Other Sources (URLs, Archives)
1. **Fetch**: Downloads or extracts the package to a cache directory using a SHA-256 hash of the URL
2. **Build**: Executes the package's build process in the cache directory (if it has a `lulu.conf.lua`)
3. **Copy**: Copies built artifacts to your project

#### Cache Directory Structure
```
~/.cache/lulu/  (or %APPDATA%/lulu on Windows)
├── abc123def456/          # Package cache (hashed directory name)
│   ├── lulu.conf.lua     # Package configuration
│   ├── main.lua          # Package source files
│   └── .lib/             # Built artifacts
│       ├── lulib/
│       │   └── mylib.lulib
│       └── dylib/
│           └── mylib.so
└── fed789cba321/          # Another package cache
    └── ...
```

#### Cache management
You can manage cache with `lulu cache` subcommand.
```bash
lulu cache list
lulu cache remove abc123def456
lulu cache clear
```

### Include
The `include` field tells lulu to include the specified `.lulib`s in your bundle. Which is the only way to include libraries resolved through [`dependencies`](#dependencies).

**Example**: 
```lua
--- keep
include = {
  "./path/to/lib.lulib",
  "@libname" -- which will be looked up in 
             -- .lib/lulib/libname.lulib` in the project root
}
```

- I advice using `@libname` to include the `dependencies` resolved to your project.


### Build
Building your `.lua` files into one bundle is the main goal of `lulu`. And you can do so either by running the `lulu bundle` command or specifiying the `build` field in your `lulu.conf.lua`. The function runs inside the build environment, where helper functions like `resolve_dependencies` and `bundle_main` are globally available.
**Example**: 
```lua
--- lulu.conf.lua
--- Here's how the build field works
build = function()
  resolve_dependencies() -- Resolves the dependencies before building

  -- you can have more operations here, like:

  -- to build the dependencies with options
  -- sets the environment variables
  set_env("MY_ENV_OPTION", true)
  set_env("MY_ENV_OPTION_2", "some value")

  -- only sets a local variable for this
  -- build instance
  set_cfg_env("MY_CFG_ENV", true)

  if not exists(".lib/lulib/some-library.lulib") then
    resolve_dependencies()

    -- automatically build sub-projects and alike:
    build("path/to/project")
    -- or manually bundle sub-projects and alike:
    bundle("path/to/main.lua", "path/to/output.lulib")
    bundle("path/to/main.lua", "path/to/output-exec")
  end

  -- including bytes into the buffer
  include_bytes("name_of_bytes_index", "path/to/file")

  -- the main bundle will be stored at
  -- `.lib/project_name` or `.lib/project_name.lulib`
  bundle_main("main.lua") -- bundles into executable
  bundle_main("main.lua", true) -- bundles into .lulib

  -- more funtions coming soon

end

--- [OUTPUT]
$ lulu build .
Resolving dependencies for /path/to/project
$
--- [/OUTPUT]
```
#### Building with CLI
```bash
lulu build .
```

#### You can also bundle with cli as:
```bash
lulu bundle ./main.lua path/to/exec
# or
lulu bundle ./init.lua path/to/lib.lulib # the .lulib is important
```

## Require
There is a custom require in lulu, for example if you have included a `.lulib` as mentioned above, then you can only import the modules from there as:
```lua
--- keep
local something = require("project_name/init") -- namespace is the project name, init or anything else works
local something_util = require("project_name/util")
```

If in the same project, you can only call the mod name as:
```lua
--- keep
local utils = require("utils")
```

Here's a simple diagram to show how it works overall:
```
@libname (in include) → .lib/lulib/libname.lulib → require("libname/module")
```

### Require Cached
You can also require URLs directly and it would import directly from the cache.
```lua
--- keep
local lib = require_cached("github:username/repo")
```

## Macros
I always(more like since i started this project) felt like lua is pretty bare bones, it's very empty and only has a limited number of features. So i added macros, inspired by the rust programming language.

Macros basically tell lulu to change code at compile time and even before building the bundle tree, therefore even letting macros modify the bundle tree as it's being built. 

### Macro types
There are two types of macros, `generating` and `transforming`.
-    **generating**: These types of macros generate new code, meaning that the final output will not only be the input code you inserted into the macro.
-    **transforming**: These types of macros only transform your code based on options, but the end result is always exactly your code.

- ⚠️ **Note**: The last argument of your macro always has to be a block with `{` and `}` with the value in the middle. 

### `cfg!`
> Transforming Macro

This macro is like compiling a portion of code conditionally, for example if you would like to have a portion of code run only on linux or only on windows:

**OS**:
```lua
--- cfg.lua
cfg! OS, {
  linux {
    print("This will only exist on linux")
  }
  windows {
    print("This will only exist on windows")
  }
}
-- you can also do:
print(cfg! OS, {
  linux {
    "This will only print on linux"
  }
  windows {
    "This will only print on windows"
  }
})

--- Custom Values:
--- You can also look for environment variables with `cfg!`, and here is how:

cfg! MY_ENV_VAR, {
  print("It is defined")
}
-- or
cfg! MY_ENV_VAR, {
  print("If defined")
}, {
  print("Not defined")
}
-- You can also look for values:
cfg! MY_ENV_VAR, {
  SOME_VALUE {
    print("IT IS SOME VALUE")
  }
  ANOTHER_VALUE {
    print("IT IS ANOTHER VALUE")
  }
  "Some Other String Value" {
    ...
  }
}, { -- optional if undefined block
  print("it is not defined")
}


--- Setting values:
--- You can set env values as such at compile time:

cfg! set, {
  MY_VAR = SOMETHING
}


--- [OUTPUT]
This will only exist on linux
This will only print on linux
Not defined
iy is not defined
--- [/OUTPUT]
```

### `match!` 
> Generating Macro

The `match!` macro lets you do a quick `switch` statement. But it also lets you check values in however condition you want.

```lua
--- match.lua
match! value, {
  "some-value" {
    print("this happens if it is some-value")
  }
  1 { -- for numbers
    print("match for numbers")
  }
  -- this one is for custom checks, val is the value of the match entry
  (val == "something") {
    print("checked for it")
  }
  _ { -- optional, if none of the above suffice
    ...
  }
}

--- If you would like to use the match! macro as a value,
--- you must use the return keyword as such:

local some_dynamic_value = match! value, {
  "case 1" {
    return "something"
  }
  _ {
    -- here you can put any code
    return ...
  }
}
```

### `for_each!`
> Generating Macro

This is a simple macro i made as a test, it basically lets you do for loops but with less code:
```lua
--- for_each.lua
for_each! item, items, {
  print(item)
}
-- translates to:
for item in ipairs(items) do 
  print(item)
end
```

### `for_pairs!`
> Generating Macro

Much like the [`for_each!`](#for_each) macro, but for pairs.
```lua
--- for_pairs.lua
for_pairs! key, value, items, {
  print(key, "=", value)
}
-- translates to:
for key, value in pairs(items) do 
  print(key, "=", value)
end
```

### `lml!`
> Generating Macro

Basically introduces `JSX` into lua, but remember it might have issues, it's still experimental. This will require you to have a function called `lml_create`.
```lua
--- lml.lua
local my_table = lml! {
  <table id="mytable">
    {
      foreach(items)(function(id)
        return <item id={f"item-{id}"} style={make_style(id)} onClick={function() print("hi") end} />
      end)
    }
  </table>
}
--- This macro rewrites the whole block into valid lua code(or at least should).

--- Here is a simple example:
local my_button = lml! {
  <box prop={value}>
    <CustomElement />
    <button id={1} name="the-name" />
  </box>
}
-- translates to
lml_create("box", { prop = value },
  lml_create(CustomElement, {}),
  lml_create("button", {
    id = 1,
    name = "the-name"
  })
)
```

- ⚠️ **Note**: The `lml_create` function does not exist, you will have to define it yourself based on your usecase.

### `import!`
> Transforming Macro

This macro lets you add files into the bundle tree at compile time, basically eliminating the need for the [`mods` field](#modules) in your `lulu.conf.lua`.

```lua
--- import.lua
import! utils, { "./utils.lua" }
import! smn, { "./src/something.lua" }
-- which will become
local smn = require("utils")
local smn = require("src-something")
```

### `include_bytes!`
> Generating Macro

This macro is basically like the `import!` macro, but instead it import files into bytes.

```lua
--- include_bytes.lua
include_bytes! text_bytes, { "./text.txt" }
print(text_bytes) -- will be the text's bytes
```


### `when!`
> Generating Macro

A simple if statement. 

```lua
--- when.lua
when! x > 5, {
  print("if true")
}
-- or
when! x > 5, {
  print("if true")
}, {
  print("if false")
}
```

### `repeat_n!`
> Generating Macro

A simple macro to repeat a task based on iteration.
```lua
--- repeat_n.lua
repeat_n! 1, 10, {
  print(i) -- prints from 1 to 10
}
```

### `try_catch!`
> Generating Macro

A simple try-catch statement.
```lua
--- try_catch.lua
try_catch! {
  -- the try block
  error("some_error")
}, { -- the catch block is optional
  -- you can use the variable err
  -- here to get the error
  print(f"Error: {err}")
}
```

### `guard!`
> Generating Macro

Checks for a condition then throws an error
```lua
--- guard.lua
guard! {
  -- has to be true, or else throws error
  is_allowed(user, page)
}, {
  "the error to throw"
}
```

### `enum!`
> Generating Macro

Creates enums that can be used with [`match!`](#match)
```lua
--- enum.lua
enum! Something, {
  Variant(content)
  EmptyVariant
}, { -- optional
  -----  Enum Methods  -----
  -- this will let you register
  -- custom functions to each 
  -- of the instances
  unwrap(item) {
    return match! item, {
      Something.Variant {
        return item.content
      }
      _ {
        return nil
      }
    }
  }
  change_content(item, value) {
    match! item, {
      Something.Variant {
        item.content =  value
      }
    }
  }
}

local value = Something.Variant("some content")

match! value, {
  Something.Variant {
    print(val.content)
  }
  Something.EmptyVariant {
    print("empty")
  }
  _ {
    print("Probably not a Something")
  }
}

-- you can also check if a value is a enum as:
Something.is(TheValue)
-- or, check if it's exactly one of the variants
Something.is(TheValue, Something.EmptyVariant)

--- you can use the functions as
local something = Something.Variant("the content")
print(something.unwrap())
```

### `class!`
> Generating Macro

Creates a simple class
```lua
--- class.lua
-- simple usecase
class! Person(name), {
  speak(){
    print(f"Hi, I am {self.name}")
  }
}

local p = Person("John")
p:speak()

-- no arguments
class! MyClass, {
  function MyClass:method()
    ...
  end
}

-- constructor
class! MyClass, {
  function MyClass:method()
    ...
  end
}, {
  print("This is the constructor")
  print(args[1])
}

-- init
class! MyClass, {
  -- the init method will be called
  -- on construction
  function MyClass:init(arg1, arg2)
    self.something = arg1
  end
}

-- inheritance
class! MyClass, {}

class! MySubClass:MyClass, {...} -- or
class! MySubClass:MyClass(...), {...}
```

**Example**:
```lua
class! Animal(name), {
  function Animal:walk()
    print(f"{self.name} is walking.")
  end
}

class! Cat:Animal(name, voice), {
  function Animal:speak()
    print(
      self.voice == "thick" and
        f"MEOWWWW"
      or
        f"meow"
    )
  end
}

local myCat = Cat("Mustard", "thick")

myCat:walk()
myCat:speak()


--- Class Decorators:
class!
@Meta({ author = "Makano", version = 1.0 })
@Register()
User:Base(),
{
  init(name, age) {
    self.name = name
    self.age = age
  }

  greet() {
    print("Hello, " .. self.name)
  }
}

--- Method Decorators:
class!
@Singleton()
Logger,
{
  log(message) {
    print("[LOG] " .. message)
  }

  @TimeIt()
  heavyTask() {
    -- some expensive computation
    local sum = 0
    for i = 1, 1_000_000 do
      sum = sum + i
    end
    return sum
  }
}

--- Parameter Decorators:
class!
@Component()
Form,
{
  submit(@Validated("string") username, @Validated("number") age) {
    print("Submitted:", username, age)
  }

  greet(@Default("Guest") name) {
    print("Hello, " .. name)
  }
}

--- Complex:
class!
@States({
  count = 0
})
@Lens()
Counter:State(),
{
  @Async()
  @Stated()
  increment(@Validated("number") step) {
    self.count = self.count + step
    print("Count is now", self.count)
  }

  @Log()
  reset() {
    self.count = 0
    print("Counter reset")
  }
}

--- For Ui or alike:
class!
@States({
  name = "something",
  id = 1
})
@Uses(Counter)
@Component()
MyWidget:Widget(props),
{
  @async()
  @stated()
  render(@Props({
    name = self.name,
    id = self.id
  }) props, @List children) {
    return lml! {
      <box name={self.name} id={self.id}>
        {children}
      </box> 
    }
  }
}

```

- ⚠️ **Note**: This feature is currently experimental. Decorators must be functions and compatible with Lua’s metatable-based classes. Multiple decorators must not conflict with each other.

### `test!`
> Transforming Macro

This macro is special. More about it at [testing](#testing).

### `Custom Macros`
> Always as Generating Macros

To define macros, you can do as such:
```lua
--- custom_macros.lua
macro {
  add_numbers ($num1, $num2, $_num3) { -- parameters with _ at the start are optional
    $num1 + $num2
  }
}
macro {
  hello ($something) {
    print("Hello, " .. $something)
  }
}
--- use as:
hello! { "World!" }
--- and:
add_numbers! 1, { 2 } -- we do this to let the macro know this is the last argument
--- or
add_numbers! { 1 }, { 2 }
```
#### Exporting macros:
You can export macros by the `macros` field in your `lulu.conf.lua` as such:
```lua
--- lulu.conf.lua
--- How to export macros:
macros = [[
macro {
  hello ($something) {
    print("Hello, " .. $something)
  }
}
macro {
  another_macro ($something) {
    ...
  }
}
]]
-- or even:
macros = io.open("_macros.lua"):read("*a")
-- you can also:
macros = {
  hello ($something) {
    print("Hello, " .. $something)
  }

  another_macro ($something) {
    ...
  }
}
```

## Lulu Environment
Lulu introduces a bunch of custom functions and globals. This environment is *sandboxed* to lulu, and therefore these functions don't come in lua by default.

### Module specific environment
```lua
--- env.lua
mod -- the current mod info, like mod name, mod config and all, 
---
mod.name -- the mod name
mod.conf -- the lulu config for this mod
current_path -- the current path
---

-- gives you either the pathname if it finds it in
-- `.lib` or just lets the OS looks it up  
lookup_dylib("SDL2.dll")

-- lets the function provided access the environment of the current module
using(function(env)
  env.something = "something"
end)
```

### Global environment
```lua
--- globals.lua
-- process
argv -- the arguments passed to the current executable
exit(1 | 0) -- exits the program
exec("command -arg", true | false) -- the output, true/false to inherit the stdout

-- FS
reads("path/to/file") -- reads as string
read("path/to/file") -- reads as bytes
exists("path/to/file") -- check if path exists
mkdir(...) -- creates dirs recursively
cp(..., ...) -- copies a single file
rename(..., ...) -- renames file
mv(..., ...) -- moves file
rm(...) -- removes/deletes file

-- Regex
re.exec("regex-pattern", "string-value") -- true/false (does it match?)
re.match("regex-pattern", "string-value") -- get match groups as array/table
-- replacing
re.replace("regex-pattern", "string-value", "$0 $1 something $2...") 
re.replace("regex-pattern", "string-value", function(all, group_1, group2)
  return group_1
end) 

-- misc
crypto.sha256('String Data') -- gives sha256sum
uuid.v4() -- gives UUIDv4
rand.from(0, 10) -- random number between
range(0, 10) -- lua table of numbers
fprint(table) -- formatted print
namespace(object, function) -- calls the function with the object as the caller

-- archive
zip.create("something.zip", {"path/to/file1", "path/to/file2"})
tar.create("something.tar.gz", {"path/to/file1", "path/to/file2"})

zip.extract("something.zip", "./path/to/extract-zip")
zip.extract("something.tar.gz", "./path/to/extract-tar")

-- fetch
net.http.request(URL, METHOD, BODY, HEADERS)

-- YAML/JSON
serde.yaml.decode("...") -- returns a lua table from yaml string
serde.yaml.encode({
  something = "value"
}) -- returns a yaml string

serde.json.decode("...") -- returns a lua table from json string
serde.json.encode({
  something = "value"
}) -- returns a json string
```

### Helper "Classes"
I added a few classes that can help you do things quicker.

#### `Vec`
You can make `Vecs` with either `vec!` macro or just `Vec({ lua_table_here })`.
- If you make it with `vec!`, you can access the properties as `.`, like `.push()`, `.insert()`, ...
- If you make it with `Vec({..})`, you can access the properties as `:`, like `:push()`, `:insert()`, ...
```lua
--- vec.lua
local vec = vec! {
  1, 2, 3, 4
} -- or
local vec = Vec({
  1, 2, 3, 4,
})
-- For this example,
-- i will be using vec!
-- you can also do Vec({...}).into()
-- to access . indexing

--- Mutation
vec.push(5) -- insert to last
vec.insert(1, 0) -- insert at (1 means the starting)
vec.sort() -- can take a sort function
vec.reverse() -- mutates
vec.pop() -- pops the last item
vec.set(5, 6) -- sets
vec.remove_at(5) -- removes that value
vec.remove(function(item, index, vec) ... end) -- removes if returns true
vec.extend(another_vec) -- merges two vecs

--- Getting
vec.len() -- get length
vec.get(1) -- gets by index
vec.find(function(item, index, vec)) -- returns the first true
vec.items -- gives you items as lua table

--- Cloning
vec.clone() -- clones
vec.map(function(item, index, vec) ... end)
vec.for_each(function(item, index, vec) ... end)
vec.filter(function(item, index, vec) ... end) -- filters all trues

--- Misc
vec:into() -- gives you the . indexing 
vec.collect() -- gives you the : indexing 
```

#### `String`
A simple string wrapper for string functions.
```lua
local string = String("String content").into() -- also follows the into

--- Cloning
string.clone()
string.split("lua pattern here") -> Vec

--- Mutating
string.push_str("some_string") -- only for lua strings
string.push_strint(String("some_string")) -- only for String

--- Getting
string.starts_with("something") -> bool
string.ends_with("something") -> bool
string.as_str() -> lua string

--- Regex
string.match("^(.+)")
string.replace("^(.+)", "$1 $2")
string.replace("^(.+)", function(all, group1) ... end)
```

#### `Map`, `Set`, `WeakMap` and `WeakSet`
Simple imlementations of `Maps` and `Sets`.
- `WeakMap` inherits `Map`
- `WeakSet` inherits `Set`
```lua
--- map_set.lua
--- Maps ---
  local map = Map():into()
  local weakmap = WeakMap():into()

  --- Cloning
  map.clone()
  weakmap.clone()

  --- Mutating
  map.set(key, value)
  map.remove(key)

  --- Getting
  map.get(key)
  map.has(key)
  map.keys()
  map.values()

--- Sets ---
  local set = Set():into()
  local weakset = WeakSet():into()

  --- Cloning
  set.clone()
  weakset.clone()

  --- Mutating
  set.add(item)
  set.remove(item)
  set.clear()

  --- Getting
  set.values()
  set.has(item)
```

#### `HashMap` and `HashSet`
`HashMap` and `HashSet` implementations made in rust. These two can not be cloned.
```lua
--- hashmap_set.lua
--- HashMap ---
local map = HashMap()

local key = {}
map:set(key, "the value")
print(map:get(key)) -- the value

map:has(key) -> bool
map:remove(key) -- removes

--- Set ---
local myset = HashSet()

myset:add("SomeValue") -- adds
myset:remove("SomeValue") -- removes and cleans memory
myset:has("SomeValue") -> bool
myset:values() -> lua table
myset:clear() -> -- removes eveyrthign
```

## Lulu Syntax
Lulu has a few sugar sprinkled to make lua tastier. Like:

### Pointers
These pointers are simulated (table references), therefore you can't really pass them to ffi without dereferencing.
```lua
--- pointers.lua
local value = 1
local ptr_to_value = &value -- or you can do ptr_of(whatever)
print(ptr_to_value) -- mem address
*value = 2 -- or you can do ptr_set(ptr, whatever)
local ptr_value = *value -- or you can do ptr_deref(ptr)
print(ptr_value)
```

### String Format
This is preprocessed at compile time by Lulu, not a native Lua syntax.
```lua
--- formatting.lua
local something = "string or whatever"
local myString = f"the string is: {something}"
-- translates to
local myString = "the string is" .. something
```

## Testing
Lulu comes with a simple system for testing. You can write tests as follows:
```lua
--- test.lua
test! {
  addition {
    -- assert failures are automatically caught and reported by the macro
    assert(1 + 1 == 2, "should be 2") -- says test succeeded
  }
  subtraction {
    assert(2 - 1 == 2, "should be 1") -- says test failed: should be 1
  }
}
```
And you can run tests as:
```bash
lulu test main.lua # or whatever file
# for specific tests:
lulu test main.lua -t subtraction
```
- ⚠️ **Note**: Keep in mind, this will only compile when testing, otherwise this portion of the code will be ejected at compile time.

## More about lulu
If you wanna help out with lulu, or wanna check out the project, [here](https://github.com/kevinJ045/lulu) is the github.

If you encounter bugs or want to contribute, check out the [issues page](https://github.com/kevinJ045/lulu/issues).