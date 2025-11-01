# KvDb

A simple key-value database using `sled` crate for quick data access.

## Methods

- **`kvdb.open`** (`string`): Open a folder for database

```lua
using { lulib.kvdb }

local db = kvdb.open("./path/to/db")

db:set("key", "value")
db:get("key")
db:remove("key")
db:contains("key")
db:id() -- generate next id
```

## KV-Table

A simple table implementation for index-based multi-value db. 

This works by having a few indexed fields from each record, and those indexes are then used for lookups.

```lua
local users = db:table("users", { "name", "age" })

users:insert({ name = "John", age = 10 })

--- Finding many values
-- Get all
users:all()
-- Find by value
users:find("name", "John")
-- Match by value (accepts regex)
users:matches("name", "^Jo") 
-- Find lt/gt
minors = users:lt("age", 18)
adults = users:gt("age", 18)

--- Finding Limits
-- by default, it only gives you 100 values.
-- you can set how much to offset and how many 
-- you want like this
users:all(limit, offset)
users:find(key, value, limit, offset)
users:matches(key, pattern, limit, offset)
users:gt(key, number, limit, offset)
users:lt(key, number, limit, offset)

-- The amount of records in table
users:length()

-- Find by id
users:index(ID)

--- Modifying
local john = users:find("name", "John")[1]
local newvalue = john

newvalue.age = 12

users:update(john.id, newvalue)
users:remove(john.id)

```
