using {
  lulib.messaging,
  lulib.crypto,
  lulib.serde
}

local broker = messaging.broker("server 127.0.0.1:5000")
local broker2 = messaging.broker("client 127.0.0.1:5000")
pin_all(broker, broker2)

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
  name = "ddd",
  id = 1
})
-- does not unpin, but terminates
all!
  broker, broker2, .stop()
;
