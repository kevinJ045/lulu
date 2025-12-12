

class!
@into_collectible("collect")
InterprocSocket(addr), {
  init(addr){
    self.__on_send = {}
    self.__on_recv = {}
    self.__recv = function() end
    self.__host = interproc.listen(addr, function(data)
      for _, r in ipairs(self.__on_recv) do
        local res = r(data)
        if res then
          data = res
        end
      end
      self.__recv(data)
    end)
  }

  on_recv(fn){
    table.insert(self.__on_recv, fn)
    return self
  }
  on_send(fn){
    table.insert(self.__on_send, fn)
    return self
  }

  listen(fn){
    self.__recv = fn
    return self
  }

  with(ext){
    if type(ext) == "table" then
      self:on_recv(ext.on_recv)
      self:on_send(ext.on_send)
    end
    return self
  }

  send(data){
    for _, r in ipairs(self.__on_send) do
      local res = r(data)
      if res then
        data = res
      end
    end
    if type(data) ~= "userdata" then
      data = ByteArray(tostring(data))
    end
    interproc.send(self.addr, data)
  }

  stop(){
    self.__host:stop()
    interproc.send(self.addr, ByteArray(""))
  }
}

interproc.JSON = {
  on_send = function(data) return serde.json.encode(data) end,
  on_recv = function(data) return serde.json.decode(data:to_string()) end,
}

interproc.String = {
  on_send = function(data) return ByteArray(data) end,
  on_recv = function(data) return data:to_string() end,
}

interproc.Serialize = {
  on_send = function(data) if data.serialize then return data:serialize() else return data end end,
}

interproc.Deserialize = function(type) return {
  on_recv = function(data) try_catch!{
    return type:deserialize(data:to_string())
  }, {
    return data
  }
  return err end,
} end
