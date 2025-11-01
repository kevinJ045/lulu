# Net Lulib

The `net` lulib is a builtin lulib that provides a bunch of tools for networking.

## HTTP

A simple http utility module to send and recieve http payload.

### HTTP Types

- **Request**:
  - **`url`** (`string`): The URL to the request source
  - **`uri`** (`string`): The pathname and query provided
  - **`method`** (`string`): The method (can be `POST`, `GET`, `DELETE`, ...)
  - **`headers`** (`table<string, string>`): The Headers
  - **`body`** (`string | bytearray`): The body content

- **Response**:
  - **`host`** (`string`): The host of the requester
  - **`uri`** (`string`): The pathname and query provided
  - **`method`** (`string`): The method (can be `POST`, `GET`, `DELETE`, ...)
  - **`headers`** (`table<string, string>`): The Headers
  - **`body`** (`bytearray`): The body content, always a [ByteArray](../reference/helper-classes.md#bytearray) result.
  - **`status`** (`number`): The status of the request response

- **PartialResponse**:
  - **`status`** (`number`): The status of the request response
  - **`headers`** (`table<string, string>`): The Headers
  - **`body`** (`function() -> bytearray`): Get the body.
  - **`json`** (`function() -> string`): Parse the json as json
  - **`yaml`** (`function() -> string`): Parse the json as yaml
  - **`into`** (`function(Class) -> Class Instance`): Parse the body as `Serializable`.
  - **`into_many`** (`function(Vec<Class>) -> Class Instance`): Parse the body as multiple entries of `Serializable`.

### HTTP Client

You can send http requests using the [reqwest crate](https://crates.io/crate/reqwest) to pull a [ByteArray](../reference/helper-classes.md#bytearray) result.

- **`net.http.request`**: `(Request) -> PartialResponse` 
- **`net.http.send`**: `(url, Request?) -> PartialResponse` 
- **`net.http.get`**: `(url, Request?) -> PartialResponse`  
- **`net.http.post`**: `(url, Request?) -> PartialResponse` 
- **`net.http.put`**: `(url, Request?) -> PartialResponse` 
- **`net.http.patch`**: `(url, Request?) -> PartialResponse` 
- **`net.http.delete`**: `(url, Request?) -> PartialResponse` 

**Example**:
```lua
using { lulib.net }

() async =>
  local res = net.http.get("https://example.com")
  print(res.status, res.body():to_string())
end
```

**Example With `Serializable`**:
```lua
using { lulib.net }

local {
  #id,
  #name,
  #username
} -> @Serializable('json') User

() async =>
  -- Get one item
  local res = net.http.get("https://jsonplaceholder.typicode.com/users/1")
  fprint(res.into(User))
  -- Get multiple items
  local res = net.http.get("https://jsonplaceholder.typicode.com/users")
  fprint(res.into_many(User))
end
```

### Http Server

A simple http listener that listens async.

- **`net.http.serve`**: `(string, function(Request) -> Response) -> nil` 

```lua
net.http.serve("0.0.0.0:8000", function(req)
  return Response {
    body = "Hello" -- will be turned to bytes in rust
  }
end)
```

## TCP

- **`net.tcp.connect`**: `(addr: string) -> { read, write, close }` 
- **`net.tcp.listen`**: `(addr: string) -> { accept() -> { read, write, close } }` 

## UDP

- **`net.tcp.bind`**: `(addr: string) -> { send_to(addr, data), recv_from(size: number) -> data }` 

## Websocket

- **`net.websocket.connect`**: `(url: string) -> { read, write, close }` 