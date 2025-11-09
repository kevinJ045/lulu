use crate::ops::{TOK_ASYNC_HANDLES, std::create_std_module};
use crossterm::{
  QueueableCommand, cursor,
  event::{Event, poll, read},
  execute,
  terminal::ClearType,
};
use std::io::stdout;

pub fn into_module() {
  create_std_module("console")
    .add_function("size", |_, ()| Ok(crossterm::terminal::size()?))
    .add_function("pos", |_, ()| Ok(cursor::position()?))
    .add_function("clear", |_, ctype: Option<String>| {
      execute!(
        stdout(),
        crossterm::terminal::Clear(match ctype.unwrap_or("".to_string()).as_str() {
          "all" => ClearType::All,
          "cl" => ClearType::CurrentLine,
          "cd" => ClearType::FromCursorDown,
          "cu" => ClearType::FromCursorUp,
          "purge" => ClearType::Purge,
          "n" => ClearType::UntilNewLine,
          _ => ClearType::All,
        })
      )?;
      Ok(())
    })
    .add_function("enter_alternate_screen", |_, ()| {
      execute!(stdout(), crossterm::terminal::EnterAlternateScreen)?;
      Ok(())
    })
    .add_function("leave_alternate_screen", |_, ()| {
      execute!(stdout(), crossterm::terminal::LeaveAlternateScreen)?;
      Ok(())
    })
    .add_function("cursor_move", |_, (x, y): (u16, u16)| {
      stdout().queue(cursor::MoveTo(x, y))?;
      Ok(())
    })
    .add_function("cursor_move_down", |_, u: u16| {
      stdout().queue(cursor::MoveDown(u))?;
      Ok(())
    })
    .add_function("cursor_move_up", |_, u: u16| {
      stdout().queue(cursor::MoveUp(u))?;
      Ok(())
    })
    .add_function("cursor_move_left", |_, u: u16| {
      stdout().queue(cursor::MoveLeft(u))?;
      Ok(())
    })
    .add_function("cursor_move_right", |_, u: u16| {
      stdout().queue(cursor::MoveRight(u))?;
      Ok(())
    })
    .add_function("cursor_to_col", |_, u: u16| {
      stdout().queue(cursor::MoveToColumn(u))?;
      Ok(())
    })
    .add_function("cursor_to_row", |_, u: u16| {
      stdout().queue(cursor::MoveToRow(u))?;
      Ok(())
    })
    .add_function("cursor_next_line", |_, u: u16| {
      stdout().queue(cursor::MoveToNextLine(u))?;
      Ok(())
    })
    .add_function("cursor_prev_line", |_, u: u16| {
      stdout().queue(cursor::MoveToPreviousLine(u))?;
      Ok(())
    })
    .add_function("cursor_hide", |_, ()| {
      stdout().queue(cursor::Hide)?;
      Ok(())
    })
    .add_function("cursor_show", |_, ()| {
      stdout().queue(cursor::Show)?;
      Ok(())
    })
    .add_function("cursor_blink", |_, blink: bool| {
      if blink {
        stdout().queue(cursor::EnableBlinking)?;
      } else {
        stdout().queue(cursor::DisableBlinking)?;
      }
      Ok(())
    })
    .add_function("poll_events", |_, func: mlua::Function| {
      TOK_ASYNC_HANDLES.lock().unwrap().push(tokio::spawn(async move {
        loop {
          match if poll(tokio::time::Duration::from_millis(500)).unwrap() {
            match read().unwrap() {
              Event::FocusGained => func.call::<bool>("focus"),
              Event::FocusLost => func.call::<bool>("unfocus"),
              Event::Paste(data) => func.call::<bool>(("paste", data)),
              Event::Key(key) => func.call::<bool>((format!("key_{:?}", key.kind).to_lowercase(), format!("{:?}", key.code))),
              Event::Mouse(mouse) => func.call::<bool>((format!("mouse_{:?}", mouse.kind).to_lowercase(), (mouse.column, mouse.row))),
              Event::Resize(x, y) => func.call::<bool>(("resize", (x, y)))
            }
          } else { Ok(false) } {
            Err(e) => eprintln!("{}", e),
            Ok(stop) => {
              if stop {
                break
              }
            }
          }
        }
      }));
      Ok(())
    })
    .on_register(|_, console_mod| Ok(console_mod))
    .into();
}
