use super::bytes::LuluByteArray;
use crate::ops::TOK_ASYNC_HANDLES;
use crate::ops::std::create_std_module;
use mlua::AnyUserData;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use whispeer::Broker;
use whispeer::async_trait::async_trait;
use whispeer::plugins::plugin::Plugin;
use whispeer::plugins::{compression::CompressionPlugin, encryption::EncryptionPlugin};

enum LuluMessagingExtension {
  Compression,
  Encryption([u8; 32]),
}
impl mlua::UserData for LuluMessagingExtension {}

struct LuaSideMessagingPlugin {
  table: mlua::Table,
  name: String,
}

#[async_trait]
impl Plugin for LuaSideMessagingPlugin {
  fn name(&self) -> &str {
    self.name.as_str()
  }

  async fn on_init(&mut self, _broker: &Broker) -> Result<(), anyhow::Error> {
    if let Ok(f) = self.table.get::<mlua::Function>("on_init") {
      f.call::<()>(()).map_err(|e| anyhow::anyhow!(e))
    } else {
      Ok(())
    }
  }

  async fn on_publish(
    &self,
    topic: &str,
    payload: &mut Vec<u8>,
    headers: &mut HashMap<String, String>,
  ) -> Result<(), anyhow::Error> {
    if let Ok(method) = self.table.get::<mlua::Function>("on_publish") {
      let (payload_new, headers_new) = method
        .call::<(Option<AnyUserData>, Option<HashMap<String, String>>)>((
          topic,
          LuluByteArray {
            bytes: payload.clone(),
          },
          headers.clone(),
        ))
        .map_err(|e| anyhow::anyhow!(e))?;

      if let Some(payload_new) = payload_new {
        *payload = payload_new
          .borrow::<LuluByteArray>()
          .map_err(|e| anyhow::anyhow!(e))?
          .bytes
          .clone();
      }

      if let Some(headers_new) = headers_new {
        *headers = headers_new;
      }

      Ok(())
    } else {
      Ok(())
    }
  }

  async fn on_message_received(
    &self,
    topic: &str,
    payload: &mut Vec<u8>,
    headers: &HashMap<String, String>,
  ) -> Result<(), anyhow::Error> {
    if let Ok(method) = self.table.get::<mlua::Function>("on_message_recieved") {
      let payload_new = method
        .call::<Option<AnyUserData>>((
          topic,
          LuluByteArray {
            bytes: payload.clone(),
          },
          headers.clone(),
        ))
        .map_err(|e| anyhow::anyhow!(e))?;

      if let Some(payload_new) = payload_new {
        *payload = payload_new
          .borrow::<LuluByteArray>()
          .map_err(|e| anyhow::anyhow!(e))?
          .bytes
          .clone();
      }

      Ok(())
    } else {
      Ok(())
    }
  }

  async fn on_before_recieved(
    &self,
    topic: &str,
    payload: &mut Vec<u8>,
    headers: &mut HashMap<String, String>,
  ) -> Result<String, anyhow::Error> {
    if let Ok(method) = self.table.get::<mlua::Function>("on_before_recieved") {
      let (topic_new, payload_new, headers_new) = method
        .call::<(
          Option<String>,
          Option<AnyUserData>,
          Option<HashMap<String, String>>,
        )>((
          topic,
          LuluByteArray {
            bytes: payload.clone(),
          },
          headers.clone(),
        ))
        .map_err(|e| anyhow::anyhow!(e))?;

      if let Some(payload_new) = payload_new {
        *payload = payload_new
          .borrow::<LuluByteArray>()
          .map_err(|e| anyhow::anyhow!(e))?
          .bytes
          .clone();
      }

      if let Some(headers_new) = headers_new {
        *headers = headers_new;
      }

      Ok(topic_new.unwrap_or(topic.to_string()))
    } else {
      Ok(topic.to_string())
    }
  }

  async fn on_subscribe(&self, topic: &str) -> Result<(), anyhow::Error> {
    if let Ok(method) = self.table.get::<mlua::Function>("on_subscribe") {
      method
        .call::<()>(topic.to_string())
        .map_err(|e| anyhow::anyhow!(e))
    } else {
      Ok(())
    }
  }
}

#[derive(Clone)]
struct LuluBroker {
  inner: Arc<Broker>,
  shutdown_tx: broadcast::Sender<()>,
}

impl LuluBroker {
  pub async fn new(addr: String) -> mlua::Result<Self> {
    let broker = Broker::start(addr).await.map_err(mlua::Error::external)?;
    let (shutdown_tx, _) = broadcast::channel(1);

    let mut rx = shutdown_tx.subscribe();

    TOK_ASYNC_HANDLES
      .lock()
      .unwrap()
      .push(tokio::spawn(async move {
        let _ = rx.recv().await;
      }));

    Ok(LuluBroker {
      inner: Arc::new(broker),
      shutdown_tx,
    })
  }
}

impl mlua::UserData for LuluBroker {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    methods.add_async_method_mut(
      "publish_async",
      |_lua, this, (topic, data): (String, mlua::Value)| async move {
        let data = match data {
          mlua::Value::String(str) => str.as_bytes().to_vec(),
          mlua::Value::UserData(ud) => ud.borrow::<LuluByteArray>()?.bytes.clone(),
          _ => Vec::new(),
        };
        this
          .inner
          .publish(topic, data)
          .await
          .map_err(mlua::Error::external)?;
        Ok(())
      },
    );

    methods.add_method_mut("stop", |_, this, ()| {
      this.shutdown_tx.send(()).map_err(mlua::Error::external)?;
      Ok(())
    });

    methods.add_method_mut(
      "subscribe",
      |_lua, this, (topic, func): (String, mlua::Function)| {
        let func = func.clone();
        this.inner.subscribe::<Vec<u8>>(topic, move |message| {
          let func = func.clone();
          Box::pin(async move {
            match func.call::<()>(LuluByteArray { bytes: message }) {
              Err(e) => {
                eprintln!("{e}");
                panic!("Aborted due to an in-thread error");
              }
              Ok(_) => {}
            }
          })
        });
        Ok(())
      },
    );

    methods.add_async_method_mut(
      "add_extension",
      |_lua, this, plugin: mlua::Value| async move {
        match plugin {
          mlua::Value::UserData(ud) => {
            if let Ok(ext) = ud.borrow::<LuluMessagingExtension>() {
              match *ext {
                LuluMessagingExtension::Encryption(key) => {
                  this
                    .inner
                    .add_plugin(EncryptionPlugin::new(key.clone()))
                    .await;
                }
                LuluMessagingExtension::Compression => {
                  this.inner.add_plugin(CompressionPlugin::new()).await;
                }
              }
            } else {
              return Err(mlua::Error::external("Extension unsupported"));
            }
          }
          mlua::Value::Table(table) => {
            let name = table.get::<String>("name").unwrap_or("unknown".to_string());
            this
              .inner
              .add_plugin(LuaSideMessagingPlugin { table, name })
              .await;
          }
          _ => return Err(mlua::Error::external("Extension unsupported")),
        };
        Ok(())
      },
    );
  }
}

pub fn into_module() {
  create_std_module("messaging")
    .add_function("compression", |_, ()| {
      Ok(LuluMessagingExtension::Compression)
    })
    .add_function("encryption", |_, key_ud: LuluByteArray| {
      let key_bytes = key_ud.bytes;
      let key = if key_bytes.len() == 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&key_bytes);
        arr
      } else {
        return Err(mlua::Error::external(format!(
          "Vec has {} bytes, expected 32",
          key_bytes.len()
        )));
      };
      Ok(LuluMessagingExtension::Encryption(key))
    })
    .on_register(|lua, mmod| {
      mmod.set(
        "broker_async",
        lua.create_async_function(|_, addr_str: String| async move {
          LuluBroker::new(addr_str).await
        })?,
      )?;

      Ok(mmod)
    })
    .add_file("messaging.lua", include_str!("../builtins/messaging.lua"))
    .depend_on("serde".to_string())
    .into();
}
