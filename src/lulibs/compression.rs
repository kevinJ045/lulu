use mlua::prelude::*;

use crate::ops::std::create_std_module;

use super::bytes::LuluByteArray;

pub fn into_module() {
  create_std_module("compression")
    .add_function(
      "compress",
      |_, (data, level): (LuaAnyUserData, Option<i32>)| {
        let bytes = data.borrow::<LuluByteArray>()?;
        let compressed = zstd::encode_all(bytes.bytes.as_slice(), level.unwrap_or(0))
          .map_err(LuaError::external)?;
        Ok(LuluByteArray { bytes: compressed })
      },
    )
    .add_function("decompress", |_, data: LuaAnyUserData| {
      let bytes = data.borrow::<LuluByteArray>()?;
      let decompressed = zstd::decode_all(bytes.bytes.as_slice()).map_err(LuaError::external)?;
      Ok(LuluByteArray {
        bytes: decompressed,
      })
    })
    .into();
}
