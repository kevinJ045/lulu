# Serde

Serde is a rust serializer and deserializer, and lulu has a very simple `serde-json` and `serde-yaml` implementation.

## Methods

- **`serde.[json/yaml].encode`** `(table)`: Encode to string
- **`serde.[json/yaml].decode`** `(string)`: Decode from string

```lua
using { lulib.serde }

fprint(serde.json.decode[[{"name": "John"}]]) 

print(serde.json.encode { name = "John" })
```

## Serialization

Once you import `serde` using `lulib.serde`, you can use the `Serializable` decorator to serialize classes automatically.

```lua
local {
  #name,
  #age
} -> @Serializable('json') Person

local p = Person {
  name = "John",
  age = 10
}

print(p:serialize()) -- encoded json string

fprint(Person:deserialize[[{"name": "John", "age": 10}]])
```

## Derialization

Once you have deserialized an object, you may want for it to have different class instances as properties, that's where `Deserializable` comes in play:

```lua
local {
  #city,
} -> @Serializable('json') Address

local {
  #name,
  #age,
  @Deserializable(Address) -- <- will ALWAYS apply
  #address
} -> @Serializable('json') Person

local p = Person {
  name = "John",
  age = 10,
  address = {
    city = "Somewhere",
  }
  -- this is the same as:
  address = Address {
    city = "Somewhere",
  }
}

-- This will have an instance of Person
-- and it's address property will be an instance
-- of Address
fprint(Person:deserialize[[{"name": "John", "age": 10, "address": {"city": "Somewhere"}}]])
```

## Multiple data

To deserialize multiple values, use `Vec`. `Vec` has `serialize` and `deserialize` by default, with built-in multivalue special functionality.

### Serialize
Serializing is simple, you just hit `serialize` and it recursively serializes.

```lua
Vec({p}):serialize()
```

### Deserialize
To deserialize a vec, you need to use `deserialize`, and also you need to parse the data yourself.

```lua
local decoded = serde.json.decode[[
  [{"name":"John","age":10,"address":{"city":"Somewhere"}}]
]]

fprint(
  Vec:deserialize(decoded, Person)
)
```

### Deserializable as `Vec`
To Deserialize as a `Vec` of something.

```lua
local {
  #city,
} -> @Serializable('json') Address

local {
  #name,
  #age,
  @Deserializable(Vec.of(Address))
  #addresses
} -> @Serializable('json') Person

local p = Person {
  name = "John",
  age = 10,
  addresses = {
    Address {
      city = "Somewhere",
    }
  }
}

fprint(p:serialize())
```