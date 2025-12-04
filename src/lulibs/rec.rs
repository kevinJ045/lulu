use std::collections::BTreeMap;

#[derive(Clone)]
pub struct LuluRec {
  pub inner: BTreeMap<String, mlua::Value>,
  frozen: bool,
}

struct IterState {
  keys: Vec<String>,
  index: usize,
  rec: LuluRec,
}

impl mlua::UserData for IterState {}

fn normalize_key(key: mlua::Value) -> mlua::Result<String> {
  match key {
    mlua::Value::String(s) => Ok(s.to_str()?.to_owned()),
    mlua::Value::Integer(i) => Ok(i.to_string()),
    mlua::Value::Number(n) => Ok(format!("{:.17}", n)),
    _ => Err(mlua::Error::RuntimeError("Unsupported key type".into())),
  }
}

impl LuluRec {
  fn new(inner: BTreeMap<String, mlua::Value>) -> Self {
    Self {
      inner,
      frozen: false,
    }
  }

  fn freeze(&mut self) {
    self.frozen = true;
  }

  fn keys(&self) -> Vec<String> {
    self.inner.keys().cloned().collect::<Vec<String>>()
  }

  fn values(&self) -> Vec<mlua::Value> {
    self.inner.values().cloned().collect::<Vec<mlua::Value>>()
  }

  fn iter(
    &self,
    lua: &mlua::Lua,
  ) -> mlua::Result<(mlua::Function, mlua::AnyUserData, mlua::Value)> {
    let keys: Vec<String> = self.inner.keys().cloned().collect();

    let state = lua.create_userdata(IterState {
      keys,
      rec: self.clone(),
      index: 0,
    })?;

    let iter_fn = lua.create_function(|lua, state: mlua::AnyUserData| {
      let mut st = state.borrow_mut::<IterState>()?;

      if st.index >= st.keys.len() {
        return Ok((mlua::Value::Nil, mlua::Value::Nil));
      }

      let key = st.keys[st.index].clone();
      st.index += 1;

      let val = st.rec.inner.get(&key).cloned().unwrap_or(mlua::Value::Nil);

      Ok((mlua::Value::String(lua.create_string(key)?), val))
    })?;

    Ok((iter_fn, state, mlua::Value::Nil))
  }
}

impl mlua::UserData for LuluRec {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_meta_method_mut(
      mlua::MetaMethod::NewIndex,
      |_, this, (key, value): (mlua::Value, mlua::Value)| {
        let key_str = normalize_key(key)?;
        this.inner.insert(key_str, value);
        Ok(())
      },
    );

    methods.add_meta_method_mut(mlua::MetaMethod::Index, |_, this, key: mlua::Value| {
      let key_str = normalize_key(key)?;

      Ok(
        this
          .inner
          .get(&key_str)
          .cloned()
          .unwrap_or(mlua::Value::Nil),
      )
    });

    methods.add_meta_method(mlua::MetaMethod::ToString, |_, _, ()| Ok("Rec {}"));

    methods.add_meta_method(mlua::MetaMethod::Call, |lua, this, ()| this.iter(lua));
    methods.add_meta_method(mlua::MetaMethod::Len, |_, this, ()| Ok(this.keys()));
    methods.add_meta_method(mlua::MetaMethod::Unm, |_, this, ()| Ok(this.values()));

    methods.add_method_mut("__freeze", |_, this, ()| Ok(this.freeze()));
  }

  fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
    fields.add_field_method_get("__len", |_, this| Ok(this.inner.len()));
  }
}

impl TryFrom<mlua::Table> for LuluRec {
  type Error = mlua::Error;
  fn try_from(tbl: mlua::Table) -> mlua::Result<Self> {
    let mut map = BTreeMap::new();

    for pair in tbl.pairs::<mlua::Value, mlua::Value>() {
      let (key, value) = pair?;

      let norm = normalize_key(key)?;
      let cloned = value.clone();

      map.insert(norm, cloned);
    }

    Ok(LuluRec::new(map))
  }
}
