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

messager.send(ByteArray('ss'))
