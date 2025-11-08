use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use mlua::Error as LuaError;

use crate::ops::std::create_std_module;


lazy_static::lazy_static! {
  pub static ref TOK_ASYNC_HANDLES: Mutex<Vec<JoinHandle<()>>> = Mutex::new(Vec::new());
}


#[derive(Clone)]
pub struct LuluThreadHandle {
  pub handle: Arc<Mutex<Option<JoinHandle<mlua::Result<mlua::Value>>>>>,
}

impl mlua::UserData for LuluThreadHandle {}

pub fn into_module(){

  create_std_module("threads")
    .on_register(|lua, threads_mod| {
      threads_mod.set(
        "spawn",
        lua.create_async_function(|lua, func: mlua::Function| async move {
          let handle = tokio::spawn(async move { func.call_async::<mlua::Value>(()).await });

          let handle = Arc::new(Mutex::new(Some(handle)));
          let handle_ref = handle.clone();

          TOK_ASYNC_HANDLES
            .lock()
            .unwrap()
            .push(tokio::spawn(async move {
              tokio::spawn(async move {
                let join_handle = {
                  let mut lock = handle.lock().unwrap();
                  lock.take()
                };

                if let Some(jh) = join_handle {
                  let _ = jh.await;
                }
              });
            }));

          Ok(lua.create_any_userdata(LuluThreadHandle { handle: handle_ref })?)
        })?,
      )?;

      // threads.join(task)
      threads_mod.set(
        "join",
        lua.create_async_function(|_, handle_ud: mlua::AnyUserData| async move {
          let handle_arc = {
            let handle = handle_ud.borrow::<LuluThreadHandle>()?;
            handle.handle.clone()
          };

          let join_handle_opt = {
            let mut opt = handle_arc.lock().unwrap();
            opt.take()
          };

          if let Some(join_handle) = join_handle_opt {
            let result = join_handle.await.map_err(LuaError::external)??;
            Ok(result)
          } else {
            Ok(mlua::Value::Nil)
          }
        })?,
      )?;

      threads_mod.set(
        "sleep",
        lua.create_async_function(|_, ms: u64| async move {
          tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
          Ok(())
        })?,
      )?;

      Ok(threads_mod)
    })
    .into();
}