use interprocess::local_socket::{GenericNamespaced, ListenerOptions, Stream, prelude::*};
use mlua::prelude::*;
use std::io::{Read, Write};

use crate::lulibs::threads::TOK_ASYNC_HANDLES;
use crate::ops::std::create_std_module;

use super::bytes::LuluByteArray;
use tokio::sync::oneshot;

pub struct PrivateMessageHost {
  stopper: Option<oneshot::Sender<()>>,
}

impl LuaUserData for PrivateMessageHost {
  fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
    methods.add_method_mut("stop", |_, this, ()| {
      if let Some(tx) = this.stopper.take() {
        let _ = tx.send(());
      }
      Ok(())
    });
  }
}

pub fn into_module() {
  create_std_module("interproc")
    .add_function("listen", |_, (addr, func): (String, LuaFunction)| {
      let addr_name = addr
        .clone()
        .to_ns_name::<GenericNamespaced>()
        .map_err(LuaError::external)?;

      let opts = ListenerOptions::new().name(addr_name);

      let listener = match opts.create_sync() {
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
          return Err(mlua::Error::external(e));
        }
        x => x?,
      };

      let (tx, mut rx) = oneshot::channel::<()>();

      TOK_ASYNC_HANDLES
        .lock()
        .unwrap()
        .push(tokio::spawn(async move {
          for client in listener.incoming() {
            if rx.try_recv().is_ok() {
              break;
            }

            if let Ok(mut stream) = client {
              let mut buf = vec![0u8; 1024];
              if let Ok(n) = stream.read(&mut buf) {
                let data = buf[..n].to_vec();
                let _ = func.call::<()>(LuluByteArray { bytes: data });
              }
            }
          }
        }));

      Ok(PrivateMessageHost { stopper: Some(tx) })
    })
    .add_function("send", |_, (addr, data_ud): (String, LuaAnyUserData)| {
      let data = data_ud.borrow::<LuluByteArray>()?;
      let bytes = data.bytes.clone();

      let addr_name = addr
        .to_ns_name::<GenericNamespaced>()
        .expect("Invalid socket name");

      let mut conn = Stream::connect(addr_name).expect("Could not connect to daemon");
      conn.write_all(bytes.as_slice()).unwrap();

      Ok(())
    })
    .add_file("interproc.lua", include_str!("../builtins/interproc.lua"))
    .depend_on("serde".to_string())
    .into();
}
