use super::bytes::LuluByteArray;
use crate::ops::std::create_std_module;
use mlua::prelude::*;

use std::sync::Arc;
use whispeer::Broker;

#[derive(Clone)]
pub struct LuluBroker {
  broker: Arc<Broker>,
}

impl LuaUserData for LuluBroker {
  fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
    methods.add_method(
      "subscribe",
      |_, this, (topic, func): (String, LuaFunction)| {
        let func = Arc::new(func);

        this.broker.subscribe::<Vec<u8>>(topic, {
          let func = Arc::clone(&func);
          move |data| {
            let func = Arc::clone(&func);
            Box::pin(async move {
              let _ = func.call::<()>(LuluByteArray { bytes: data });
            })
          }
        });

        Ok(())
      },
    );

    methods.add_async_method(
      "publish",
      async |_, this, (topic, data_ud): (String, LuaAnyUserData)| {
        let data = data_ud.borrow::<LuluByteArray>()?;
        let bytes = data.bytes.clone();

        this
          .broker
          .publish(topic, bytes)
          .await
          .map_err(LuaError::external)?;

        Ok(())
      },
    );
  }
}

pub fn into_module() {
  create_std_module("messaging")
    .on_register(|lua, mmod| {
      mmod.set(
        "broker",
        lua.create_async_function(async |lua, addr: String| {
          let wroker = Broker::start(addr).await.expect("Failed to start broker A");
          let broker = LuluBroker {
            broker: Arc::new(wroker),
          };

          lua.create_userdata(broker)
        })?,
      )?;

      Ok(mmod)
    })
    .into();
}
