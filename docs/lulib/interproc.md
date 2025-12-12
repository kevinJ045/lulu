# Interproc

A simple interprocess communication library for lulu.

## Example

**Basic Example**:
```lua
using {
  lulib.interproc
}

messager = InterprocSocket("mysoc")
  :listen(function(message) print(message:to_string()) end)

messager.send(ByteArray("data"))
```

**JSON messaging**:
```lua
using {
  lulib.interproc,
  lulib.serde
}

messager = InterprocSocket("mysoc"):into()
  .with(interproc.JSON)
  .listen(function(message) fprint(message) end)

messager.send({
  event = "some-event",
  data = "some-data"
})

messager.stop()
```

**Serialized Data**:
```lua
using {
  lulib.interproc,
  lulib.serde
}

local {
  Stop,
  Start,
  Task(task)
} -< @Serializable('json') Event

local {
  @Deserializable(Event)
  #event,
  #data 
} -> @Serializable('json') Message

messager = InterprocSocket("mysoc"):into()
  .with(interproc.Serialize)
  .with(interproc.Deserialize(Message))
  .listen(function(message) fprint(message) end)

messager.send(Message {
  event = Event.Start,
  -- only string data is allowed
  -- when serializing
  data = "some-string-data"
})

```

**Custom Extensions**:
```lua
using {
  lulib.interproc
}

messager = InterprocSocket("mysoc")
  :with({ -- Stringify responses
    on_recv = function(data) return data:to_string() end,
    on_send = function(string) return ByteArray(string) end,
  })
  :listen(function(message) print(message) end)

messager:send("data")

--- Note: You can also do :with(interproc.String)
```
