
#[derive(Clone)]
pub struct LuluByteArray {
  pub bytes: Vec<u8>,
}

impl mlua::UserData for LuluByteArray {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method("to_table", |_, this, ()| Ok(this.bytes.clone()));
    methods.add_method("len", |_, this, ()| Ok(this.bytes.len()));
    methods.add_method("to_hex", |_, this, ()| {
      Ok(
        this
          .bytes
          .iter()
          .map(|b| format!("{:02x}", b))
          .collect::<String>(),
      )
    });

    methods.add_method("to_string", |_lua, this, encoding: Option<String>| {
      let enc_name = encoding.unwrap_or_else(|| "utf-8".to_string());
      match enc_name.to_lowercase().as_str() {
        "utf-8" => Ok(String::from_utf8_lossy(&this.bytes).to_string()),
        _ => Err(mlua::Error::RuntimeError(format!(
          "Unsupported encoding '{}'",
          enc_name
        ))),
      }
    });

    methods.add_method_mut("extend", |_, this, other: mlua::AnyUserData| {
      let other_bytes = other.borrow::<LuluByteArray>()?;
      this.bytes.extend(&other_bytes.bytes);
      Ok(())
    });

    methods.add_method_mut("extend_table", |_, this, other: Vec<u8>| {
      this.bytes.extend(other);
      Ok(())
    });

    methods.add_method_mut("push", |_, this, byte: u8| {
      this.bytes.push(byte);
      Ok(())
    });

    methods.add_method_mut("pop", |_, this, ()| Ok(this.bytes.pop()));

    methods.add_method_mut("clear", |_, this, ()| {
      this.bytes.clear();
      Ok(())
    });

    methods.add_method("slice", |_, this, (start, stop): (usize, usize)| {
      let start = start.saturating_sub(1);
      let stop = stop.min(this.bytes.len());
      Ok(LuluByteArray {
        bytes: this.bytes[start..stop].to_vec(),
      })
    });

    methods.add_method("copy", |_, this, ()| {
      Ok(LuluByteArray {
        bytes: this.bytes.clone(),
      })
    });

    methods.add_method("new", |_, _, bytes: Vec<u8>| Ok(LuluByteArray { bytes }));

    methods.add_method("map", |_, this, func: mlua::Function| {
      let mapped = this
        .bytes
        .iter()
        .map(|b| func.call::<u8>(*b).unwrap_or(*b))
        .collect();
      Ok(LuluByteArray { bytes: mapped })
    });
  }
}
