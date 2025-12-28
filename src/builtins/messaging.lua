

local class! Broker;

messaging.broker = function(addr)
  local proxy_broker = Broker()
  local extensions = Vec():into()
  local broker = sync_call(messaging.broker_async, addr)

  function proxy_broker.publish(topic, payload)
    extensions.for_each(function(ext)
      if not ext.on_publish then return end
      payload = ext.on_publish(topic, payload)
    end)
    return sync_call(broker.publish_async, broker, topic, payload)
  end

  function proxy_broker.subscribe(topic, func)
    return broker:subscribe(topic, function(data)
      extensions.for_each(function(ext)
        if not ext.on_message_received then return end
        data = ext.on_message_received(topic, data)
      end)
      func(data)
    end)
  end

  function proxy_broker.with(ext)
    local plug = ext
    if type(ext) == "table" then
      if ext.is_local then
        extensions.push(ext)
        return nil
      else
        if ext.on_init then
          local on_init = ext.on_init
          ext.on_init = function()
            return on_init(proxy_broker)
          end
        end
        plug = ext
      end
    end
    broker:add_extension(plug)
    return proxy_broker
  end


  function proxy_broker.stop()
    broker:stop()
  end

  return proxy_broker
end

local class! @into_collectible('__') PluginBuilder(ext), {
  init(){
    self.ext.is_local = false
  }
  loc(){
    self.ext.is_local = true
    return self
  }
  on(name, func){
    local ext = self.ext
    self.ext[f"on_{name}"] = function(...)
      return func(ext, ...)
    end
    return self
  }
  build(){
    return self.ext
  }
}

messaging.plugin = function(name)
  return PluginBuilder(collect! { name }):into()
end


messaging.stringify = messaging.plugin('StringPlugin')
  .loc()
  .on('publish', function(self, topic, message)
    return ByteArray(message)
  end)
  .on('message_received', function(self, topic, message)
    return message:to_string()
  end)
  .build()

messaging.json = messaging.plugin('JSONPlugin')
  .loc()
  .on('publish', function(self, topic, message)
    return ByteArray(serde.json.encode(message))
  end)
  .on('message_received', function(self, topic, message)
    return serde.json.decode(message:to_string())
  end)
  .build()

messaging.yaml = messaging.plugin('JSONPlugin')
  .loc()
  .on('publish', function(self, topic, message)
    return ByteArray(serde.yaml.encode(message))
  end)
  .on('message_received', function(self, topic, message)
    return serde.yaml.decode(message:to_string())
  end)
  .build()


messaging.Serialize = messaging.plugin('SerializePlugin')
  .loc()
  .on('publish', function(_, _, data) if data.serialize then return ByteArray(data:serialize()) else return data end end)
  .build()

messaging.Deserialize = function(type) return messaging.plugin('DeserializePlugin')
  .loc()
  .on('message_received', function(_, _, data) try_catch!{
      return type:deserialize(data:to_string())
    }, {
      return data
    }
    return err
  end)
  .build()
end
