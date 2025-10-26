
local class! HttpMetadata(
  @default_to({})
  #headers,
  #body,
  @default_to("")
  #url,
  @default_to("")
  #uri,
  @default_to("GET")
  #method,
);

class! Request:HttpMetadata;
class! Response:HttpMetadata;

local function _handle_response(response)
  local body = response.body
  local headers = response.headers or {}
  local status = response.status or 0

  local res = {
    status = status,
    headers = headers,
  }

  res.text = function()
    if type(body) == "string" then
      return body
    elseif type(body) == "userdata" then
      return body:to_string()
    end
    return tostring(body)
  end

  res.json = function()
    local text = res.text()
    local ok, decoded = pcall(serde.json.decode, text)
    if not ok then
      error("Invalid JSON: " .. tostring(decoded))
    end
    return decoded
  end

  res.into_many = function(_class)
    local params = {}
    try_catch! {
      params = res.json()
    }, {
      try_catch! {
        params = res.yaml()
      }, {
        params = {}
      }
    }

    local items = Vec()
    for k, v in pairs(params) do
      items:push(_class(v))
    end

    return items
  end

  res.into = function(_class)
    if not _class then return end

    if _class.deserialize then
      return _class:deserialize(res.text())
    else
      local params = {}
      try_catch! {
        params = res.json()
      }, {
        try_catch! {
          params = res.yaml()
        }, {
          params = {}
        }
      }

      return _class(params)
    end
  end

  res.yaml = function()
    local text = res.text()
    local ok, decoded = pcall(serde.yaml.decode, text)
    if not ok then
      error("Invalid YAML: " .. tostring(decoded))
    end
    return decoded
  end

  res.body = function()
    return body
  end

  return res
end

macro {
  request_method($method, $method_string){
    net.http.$method = function(url, options)
      local data = options or {}
      local response = net.http.request {
        method = data.method or $method_string,
        url = url,
        headers = data.headers or {},
        body = data.body,
      }
      return _handle_response(response)
    end
  }
}

request_method! send, "GET";
request_method! get, "GET";
request_method! post, "POST";
request_method! patch, "PATCH";
request_method! put, "PUT";
request_method! delete, "DELETE";