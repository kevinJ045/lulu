# Crypto

`aes_gcm` bindings for lulu.

## Methods

- **`crypto.encrypt(key, nonce, data: ByteArray)`**: Returns an encrypted `ByteArray` of supplied data.
- **`crypto.encrypt(key, nonce, data: ByteArray)`**: Decrypts a provided encrypted `ByteArray`.
- **`crypto.random_key(size)`**: Provides a `ByteArray` of a key.
- **`crypto.random_nonce(size)`**: Provides a `ByteArray` of a nonce.


```lua
using { lulib.crypto }

local key = crypto.random_key()
local nonce = crypto.random_nonce()

local entity = ByteArray("Data")

local encrypted = crypto.encrypt(key, nonce, entity)

local decrypted = try!
  return Ok(crypto.decrypt(key, nonce, encrypted));

match(decrypted) do
  if Err then
    print("Decryption Failed")
  if Ok then
    local res = decrypted.unwrap():to_string()
    print("Result: \"" .. res .. "\"")
end
```
