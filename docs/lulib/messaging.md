# Messaging

A simple pub/sub messaging library for lulu.

## Example

```lua
-- server.lua
using {
  lulib.messaging
}

broker = messaging.broker("server 127.0.0.1:5000")

broker.subscribe("mytopic", function(data)
  fprint("Received: ", data:to_string())
end)

-- client.lua
using {
  lulib.messaging
}

local broker = messaging.broker("client 127.0.0.1:5000")

broker.publish("mytopic", ByteArray("Hello!"))
```

## Methods:

- **`messaging.broker("client/server ADDR")`**: Creates a broker instance.
    - **`broker.publish(topic: string, byte_array | payload)`**: Publishes data to other brokers(the payload is `ByteArray` by default).
    - **`broker.subscribe(topic: string, fun(byte_array | payload))`**: Calls the subscribed function when other brokers `publish`.
    - **`broker.with(plugin)`**: Attaches the plugin provided.
    - **`broker.stop()`**: Stops the channel.
- **`messaging.plugin(name: string)`**: A simple plugin builder.
    - **`pluginbuilder.on(topic: string, fun(plugin, topic, payload?, headers?))`**: Modifies payload and/or topic/headers.
    - **`pluginbuilder.loc()`**: Switches plugin to local(lua-side) mode.
    - **`pluginbuilder.build()`**: Returns the final plugin result.
  

## Plugins

There are two types of messaging plugins.

### Casted
Casted plugins are casted into the rust side, and their order can also matter among other casted plugins as well as having access to headers.

```lua
local my_custom_plugin = messaging.plugin('my_plugin')
  .on('publish', function(_, topic, payload, headers)
    print(payload:to_string())
    -- if you modified anything:
    return payload, headers
  end)
  .build()

broker.with(messaging.compression()) -- runs first on publish, last on recieve
broker.with(my_custom_plugin) -- runs after compression on publish, after on recieve
broker.with(messaging.encryption(key)) -- runs last on publish, first on recieve
```

### Local
Local plugins run on the lua side before the data is transferred into the rust side, can be used to serialize/deserialize the payload.

```lua

local my_local_plugin = messaging.plugin('my_local_plugin')
  .loc()
  .on('publish', function(_, topic, payload)
    -- immediate payload, if you passed a string, you get a string here
    print(payload)
    -- always needed
    return ByteArray(payload)
  end)
  .build()

broker.with(my_local_plugin) -- always runs before publish and after recieve
broker.with(messaging.compression())
broker.with(my_custom_plugin)
broker.with(messaging.encryption(key))
```

### Plugin Events:
- `publish(topic, payload, headers) -> payload, headers`: Happens on publish
- `init(broker)`: When the broker initiates
- `message_received(topic, payload, headers) -> payload`: Modify recieved payload
- `before_recieved(topic, payload, headers) -> topic, payload, headers`: Modify before recieve (only on casted)
- `subscribe(topic)`: Happens when subscription happens (only on casted)

### Simple Stringify Plugin:

```lua
using {
  lulib.messaging
}

local broker = messaging.broker("server 127.0.0.1:5000")
local broker2 = messaging.broker("client 127.0.0.1:5000")

broker.with(messaging.stringify)
broker2.with(messaging.stringify)

broker.subscribe("mytopic", function(data)
  fprint("Received: ", data)
end)

broker2.publish("mytopic", "A sample String")
```

### Simple JSON Plugin:

```lua
using {
  lulib.messaging
}

local broker = messaging.broker("server 127.0.0.1:5000")
local broker2 = messaging.broker("client 127.0.0.1:5000")

broker.with(messaging.json)
broker2.with(messaging.json)

broker.subscribe("my-json-data", function(data)
  fprint("Received: ", data)
end)

broker2.publish("my-json-data", {
  data = "some-data"
})
```
**Note**: Works the same with `messaging.yaml`.

### Serializer/Deserializer Plugin:

```lua
using {
  lulib.messaging
}

local {
  #name,
  #id
} -> @Serializable('json') User

local broker = messaging.broker("server 127.0.0.1:5000")
local broker2 = messaging.broker("client 127.0.0.1:5000")

broker.with(messaging.Deserialize(User))
broker2.with(messaging.Serialize)
-- you can also add both the deserializer and serializer
-- into both brokers

broker.subscribe("user", function(data)
  fprint("Received: ", data)
end)

broker2.publish("user", User {
  name = "john",
  id = 1
})
```

### Compression

```lua
using {
  lulib.messaging
}

local broker = messaging.broker("server 127.0.0.1:5000")
local broker2 = messaging.broker("client 127.0.0.1:5000")

broker.with(messaging.compression())
broker2.with(messaging.compression())

broker.subscribe("send-something", function(data)
  fprint("Received: ", data:to_string())
end)

broker2.publish("send-something", ByteArray("This will be compressed"))
```

### Encryption

```lua
using {
  lulib.messaging,
  lulib.crypto
}

local key = crypto.random_key()

local broker = messaging.broker("server 127.0.0.1:5000")
local broker2 = messaging.broker("client 127.0.0.1:5000")

broker.with(messaging.encryption(key))
broker2.with(messaging.encryption(key))

broker.subscribe("send-something", function(data)
  fprint("Received: ", data:to_string())
end)

broker2.publish("send-something", ByteArray("This will be encrypted"))
```

## Complex Example:

```lua
using {
  lulib.messaging,
  lulib.crypto,
  lulib.serde
}

local broker = messaging.broker("server 127.0.0.1:5000")
local broker2 = messaging.broker("client 127.0.0.1:5000")
-- pin to keep alive
pin_all(broker, broker2)

-- serializable data
local {
  #name,
  #id
} -> @Serializable('json') User

local k1 = crypto.random_key()

-- The order of application matters,
-- both server and client must have the same order
broker.with(messaging.Deserialize(User))
broker.with(messaging.encryption(k1))
broker.with(messaging.compression())

broker2.with(messaging.Serialize)
broker2.with(messaging.encryption(k1))
broker2.with(messaging.compression())

broker.subscribe("mytopic", function(data)
  fprint("Received: ", data)
end)

broker2.publish("mytopic", User {
  name = "john",
  id = 1
})

-- does not unpin, but terminates
all!
  broker, broker2, .stop()
;
```
