use mlua::Lua;

use crate::{lulu::Lulu, ops};

#[allow(unused)]
pub const FLAVOR_ID_BASE: u16 = 1000;
#[allow(unused)]
pub const FLAVOR_ID_UI: u16 = 2000;
#[allow(unused)]
pub const FLAVOR_ID_SERVE: u16 = 3000;
#[allow(unused)]
pub const FLAVOR_ID_GENERAL: u16 = 9999;

pub const fn current_flavor_id() -> u16 {
  #[cfg(feature = "ui")]
  {
    return FLAVOR_ID_UI;
  }
  #[cfg(feature = "serve")]
  {
    return FLAVOR_ID_SERVE;
  }
  #[cfg(feature = "general")]
  {
    return FLAVOR_ID_GENERAL;
  }

  #[allow(unused)]
  FLAVOR_ID_BASE
}

pub fn current_flavor_scripts(_: &Lua, _: &Lulu) -> mlua::Result<()> {
  Ok(())
}

pub fn current_flavor_ops(lua: &Lua, lulu: &Lulu) -> mlua::Result<()> {
  ops::register_ops(&lua, lulu)?;


  #[cfg(feature = "ui")]
  crate::ui::register;

  Ok(())
}
