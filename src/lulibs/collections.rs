use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct LuluHashSet {
  pub items: HashSet<usize>, // store RegistryKey pointers as usize
}

impl mlua::UserData for LuluHashSet {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method_mut("add", |lua, this, value: mlua::Value| {
      let key = lua.create_registry_value(value)?;
      this.items.insert(Box::into_raw(Box::new(key)) as usize);
      Ok(())
    });

    methods.add_method_mut("remove", |lua, this, value: mlua::Value| {
      let mut to_remove = None;
      for &ptr in &this.items {
        let key_ref = unsafe { &*(ptr as *mut mlua::RegistryKey) };
        let v = lua.registry_value::<mlua::Value>(key_ref)?;
        if v == value {
          to_remove = Some(ptr);
          break;
        }
      }
      if let Some(ptr) = to_remove {
        this.items.remove(&ptr);
        drop(unsafe { Box::from_raw(ptr as *mut mlua::RegistryKey) }); // drop the key
      }
      Ok(())
    });

    methods.add_method("has", |lua, this, value: mlua::Value| {
      for &ptr in &this.items {
        let key_ref = unsafe { &*(ptr as *mut mlua::RegistryKey) };
        let v = lua.registry_value::<mlua::Value>(key_ref)?;
        if v == value {
          return Ok(true);
        }
      }
      Ok(false)
    });

    methods.add_method("values", |lua, this, _: ()| {
      let tbl = lua.create_table()?;
      for (i, &ptr) in this.items.iter().enumerate() {
        let key_ref = unsafe { &*(ptr as *mut mlua::RegistryKey) };
        let value = lua.registry_value::<mlua::Value>(key_ref)?;
        tbl.set(i + 1, value)?;
      }
      Ok(tbl)
    });

    methods.add_method_mut("clear", |_, this, _: ()| {
      for &ptr in &this.items {
        drop(unsafe { Box::from_raw(ptr as *mut mlua::RegistryKey) });
      }
      this.items.clear();
      Ok(())
    });
  }
}

#[derive(Clone)]
pub struct LuluHashMap {
  pub items: HashMap<usize, usize>, // key ptr → value ptr
}

impl mlua::UserData for LuluHashMap {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_method_mut(
      "set",
      |lua, this, (key, value): (mlua::Value, mlua::Value)| {
        let key_ptr = Box::into_raw(Box::new(lua.create_registry_value(key)?)) as usize;
        let value_ptr = Box::into_raw(Box::new(lua.create_registry_value(value)?)) as usize;
        this.items.insert(key_ptr, value_ptr);
        Ok(())
      },
    );

    methods.add_method("get", |lua, this, key: mlua::Value| {
      for (&k_ptr, &v_ptr) in &this.items {
        let k_ref = unsafe { &*(k_ptr as *mut mlua::RegistryKey) };
        let k_val = lua.registry_value::<mlua::Value>(k_ref)?;
        if k_val == key {
          let v_ref = unsafe { &*(v_ptr as *mut mlua::RegistryKey) };
          return Ok(lua.registry_value::<mlua::Value>(v_ref)?);
        }
      }
      Ok(mlua::Value::Nil)
    });

    methods.add_method("has", |lua, this, key: mlua::Value| {
      for (&k_ptr, _) in &this.items {
        let k_ref = unsafe { &*(k_ptr as *mut mlua::RegistryKey) };
        if lua.registry_value::<mlua::Value>(k_ref)? == key {
          return Ok(true);
        }
      }
      Ok(false)
    });

    methods.add_method_mut("remove", |lua, this, key: mlua::Value| {
      let mut to_remove = None;
      for (&k_ptr, &v_ptr) in &this.items {
        let k_ref = unsafe { &*(k_ptr as *mut mlua::RegistryKey) };
        if lua.registry_value::<mlua::Value>(k_ref)? == key {
          to_remove = Some((k_ptr, v_ptr));
          break;
        }
      }
      if let Some((k_ptr, v_ptr)) = to_remove {
        this.items.remove(&k_ptr);
        drop(unsafe { Box::from_raw(k_ptr as *mut mlua::RegistryKey) });
        drop(unsafe { Box::from_raw(v_ptr as *mut mlua::RegistryKey) });
      }
      Ok(())
    });
  }
}
