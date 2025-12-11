
using {
  lulib.crypto
}

local k1 = crypto.random_key()
local k2 = crypto.random_key()

local n1 = crypto.random_nonce()

local something = ByteArray("something")

local encrypted = crypto.encrypt(k1, n1, something)

local decrypted = try!
  return Ok(crypto.decrypt(k1, n1, encrypted));

local decrypted_error = try!
  return Ok(crypto.decrypt(k2, n1, encrypted));

match(decrypted_error) do
  if Err then
    print("Intentional Decryption Failure")
  if Ok then
    print("Should have failed")
end

match(decrypted) do
  if Err then
    print("Decryption Failed")
  if Ok then
    local res = decrypted.unwrap():to_string()
    print("Result: \"" .. res .. "\"")
end
