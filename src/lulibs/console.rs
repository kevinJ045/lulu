use crate::ops::{TOK_ASYNC_HANDLES, std::create_std_module};
use crossterm::{
  QueueableCommand, cursor, event::{Event, poll, read}, execute, style::{self, Color, StyledContent, Stylize}, terminal::ClearType
};
use std::io::stdout;

#[derive(Clone)]
pub struct ConsoleString {
  pub string: StyledContent<String>
}

impl mlua::UserData for ConsoleString {
  fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
    
    methods.add_method_mut("blue", |_, this, ()| {
      this.string = this.string.clone().blue();
      Ok(())
    });    
    methods.add_method_mut("black", |_, this, ()| {
      this.string = this.string.clone().black();
      Ok(())
    });
    methods.add_method_mut("cyan", |_, this, ()| {
      this.string = this.string.clone().cyan();
      Ok(())
    });
    methods.add_method_mut("green", |_, this, ()| {
      this.string = this.string.clone().green();
      Ok(())
    });
    methods.add_method_mut("grey", |_, this, ()| {
      this.string = this.string.clone().grey();
      Ok(())
    });
    methods.add_method_mut("magenta", |_, this, ()| {
      this.string = this.string.clone().magenta();
      Ok(())
    });
    methods.add_method_mut("red", |_, this, ()| {
      this.string = this.string.clone().red();
      Ok(())
    });
    methods.add_method_mut("yellow", |_, this, ()| {
      this.string = this.string.clone().yellow();
      Ok(())
    });
    methods.add_method_mut("white", |_, this, ()| {
      this.string = this.string.clone().white();
      Ok(())
    });
    methods.add_method_mut("rgb", |_, this, color: mlua::Table| {
      this.string = this.string.clone().with(Color::Rgb { r: color.get(1)?, g: color.get(2)?, b: color.get(3)? });
      Ok(())
    });
    methods.add_method_mut("on_rgb", |_, this, color: mlua::Table| {
      this.string = this.string.clone().on(Color::Rgb { r: color.get(1)?, g: color.get(2)?, b: color.get(3)? });
      Ok(())
    });

    methods.add_method_mut("dim", |_, this, ()| {
      this.string = this.string.clone().dim();
      Ok(())
    });
    methods.add_method_mut("dark_cyan", |_, this, ()| {
      this.string = this.string.clone().dark_cyan();
      Ok(())
    });
    methods.add_method_mut("dark_green", |_, this, ()| {
      this.string = this.string.clone().dark_green();
      Ok(())
    });
    methods.add_method_mut("dark_grey", |_, this, ()| {
      this.string = this.string.clone().dark_grey();
      Ok(())
    });
    methods.add_method_mut("dark_magenta", |_, this, ()| {
      this.string = this.string.clone().dark_magenta();
      Ok(())
    });
    methods.add_method_mut("dark_red", |_, this, ()| {
      this.string = this.string.clone().dark_red();
      Ok(())
    });
    methods.add_method_mut("dark_yellow", |_, this, ()| {
      this.string = this.string.clone().dark_yellow();
      Ok(())
    });
    
    // Styles
    methods.add_method_mut("hidden", |_, this, ()| {
      this.string = this.string.clone().hidden();
      Ok(())
    });
    methods.add_method_mut("reset", |_, this, ()| {
      this.string = this.string.clone().reset();
      Ok(())
    });
    methods.add_method_mut("negative", |_, this, ()| {
      this.string = this.string.clone().negative();
      Ok(())
    });
    methods.add_method_mut("slow_blink", |_, this, ()| {
      this.string = this.string.clone().slow_blink();
      Ok(())
    });
    methods.add_method_mut("rapid_blink", |_, this, ()| {
      this.string = this.string.clone().rapid_blink();
      Ok(())
    });
    methods.add_method_mut("italic", |_, this, ()| {
      this.string = this.string.clone().italic();
      Ok(())
    });
    methods.add_method_mut("bold", |_, this, ()| {
      this.string = this.string.clone().bold();
      Ok(())
    });
    methods.add_method_mut("underline", |_, this, color: mlua::Table| {
      this.string = this.string.clone().underline(Color::Rgb { r: color.get(1)?, g: color.get(2)?, b: color.get(3)? });
      Ok(())
    });
    methods.add_method_mut("underline_white", |_, this, ()| {
      this.string = this.string.clone().underline_white();
      Ok(())
    });
    methods.add_method_mut("underline_blue", |_, this, ()| {
      this.string = this.string.clone().underline_blue();
      Ok(())
    });
    methods.add_method_mut("underline_black", |_, this, ()| {
      this.string = this.string.clone().underline_black();
      Ok(())
    });
    methods.add_method_mut("underline_cyan", |_, this, ()| {
      this.string = this.string.clone().underline_cyan();
      Ok(())
    });
    methods.add_method_mut("underline_green", |_, this, ()| {
      this.string = this.string.clone().underline_green();
      Ok(())
    });
    methods.add_method_mut("underline_grey", |_, this, ()| {
      this.string = this.string.clone().underline_grey();
      Ok(())
    });
    methods.add_method_mut("underline_magenta", |_, this, ()| {
      this.string = this.string.clone().underline_magenta();
      Ok(())
    });
    methods.add_method_mut("underline_red", |_, this, ()| {
      this.string = this.string.clone().underline_red();
      Ok(())
    });
    methods.add_method_mut("underline_yellow", |_, this, ()| {
      this.string = this.string.clone().underline_yellow();
      Ok(())
    });
    methods.add_method_mut("underline_dark_cyan", |_, this, ()| {
      this.string = this.string.clone().underline_dark_cyan();
      Ok(())
    });
    methods.add_method_mut("underline_dark_green", |_, this, ()| {
      this.string = this.string.clone().underline_dark_green();
      Ok(())
    });
    methods.add_method_mut("underline_dark_grey", |_, this, ()| {
      this.string = this.string.clone().underline_dark_grey();
      Ok(())
    });
    methods.add_method_mut("underline_dark_magenta", |_, this, ()| {
      this.string = this.string.clone().underline_dark_magenta();
      Ok(())
    });
    methods.add_method_mut("underline_dark_red", |_, this, ()| {
      this.string = this.string.clone().underline_dark_red();
      Ok(())
    });
    methods.add_method_mut("underline_dark_yellow", |_, this, ()| {
      this.string = this.string.clone().underline_dark_yellow();
      Ok(())
    });
    
    methods.add_method_mut("on_white", |_, this, ()| {
      this.string = this.string.clone().on_white();
      Ok(())
    });
    methods.add_method_mut("on_blue", |_, this, ()| {
      this.string = this.string.clone().on_blue();
      Ok(())
    });
    methods.add_method_mut("on_black", |_, this, ()| {
      this.string = this.string.clone().on_black();
      Ok(())
    });
    methods.add_method_mut("on_cyan", |_, this, ()| {
      this.string = this.string.clone().on_cyan();
      Ok(())
    });
    methods.add_method_mut("on_green", |_, this, ()| {
      this.string = this.string.clone().on_green();
      Ok(())
    });
    methods.add_method_mut("on_grey", |_, this, ()| {
      this.string = this.string.clone().on_grey();
      Ok(())
    });
    methods.add_method_mut("on_magenta", |_, this, ()| {
      this.string = this.string.clone().on_magenta();
      Ok(())
    });
    methods.add_method_mut("on_red", |_, this, ()| {
      this.string = this.string.clone().on_red();
      Ok(())
    });
    methods.add_method_mut("on_yellow", |_, this, ()| {
      this.string = this.string.clone().on_yellow();
      Ok(())
    });
    methods.add_method_mut("on_dark_cyan", |_, this, ()| {
      this.string = this.string.clone().on_dark_cyan();
      Ok(())
    });
    methods.add_method_mut("on_dark_green", |_, this, ()| {
      this.string = this.string.clone().on_dark_green();
      Ok(())
    });
    methods.add_method_mut("on_dark_grey", |_, this, ()| {
      this.string = this.string.clone().on_dark_grey();
      Ok(())
    });
    methods.add_method_mut("on_dark_magenta", |_, this, ()| {
      this.string = this.string.clone().on_dark_magenta();
      Ok(())
    });
    methods.add_method_mut("on_dark_red", |_, this, ()| {
      this.string = this.string.clone().on_dark_red();
      Ok(())
    });
    methods.add_method_mut("on_dark_yellow", |_, this, ()| {
      this.string = this.string.clone().on_dark_yellow();
      Ok(())
    });


    methods.add_method_mut("to_string", |_, this, ()| {
      Ok(this.string.clone().to_string())
    });
  }
}

pub fn into_module() {
  create_std_module("console")
    // String
    .add_function("string", |_, thing: String| Ok(ConsoleString { string: thing.stylize() }))
    
    // Colors
    .add_function("blue", |_, thing: String| Ok(thing.blue().to_string()))
    .add_function("black", |_, thing: String| Ok(thing.black().to_string()))
    .add_function("cyan", |_, thing: String| Ok(thing.cyan().to_string()))
    .add_function("green", |_, thing: String| Ok(thing.green().to_string()))
    .add_function("grey", |_, thing: String| Ok(thing.grey().to_string()))
    .add_function("magenta", |_, thing: String| Ok(thing.magenta().to_string()))
    .add_function("red", |_, thing: String| Ok(thing.red().to_string()))
    .add_function("yellow", |_, thing: String| Ok(thing.yellow().to_string()))
    .add_function("white", |_, thing: String| Ok(thing.white().to_string()))
    
    .add_function("rgb", |_, (thing, color): (String, mlua::Table)| Ok(thing.with(Color::Rgb { r: color.get(1)?, g: color.get(2)?, b: color.get(3)? }).to_string()))
    .add_function("on_rgb", |_, (thing, color): (String, mlua::Table)| Ok(thing.on(Color::Rgb { r: color.get(1)?, g: color.get(2)?, b: color.get(3)? }).to_string()))
    
    .add_function("dim", |_, thing: String| Ok(thing.dim().to_string()))
    .add_function("dark_cyan", |_, thing: String| Ok(thing.dark_cyan().to_string()))
    .add_function("dark_green", |_, thing: String| Ok(thing.dark_green().to_string()))
    .add_function("dark_grey", |_, thing: String| Ok(thing.dark_grey().to_string()))
    .add_function("dark_magenta", |_, thing: String| Ok(thing.dark_magenta().to_string()))
    .add_function("dark_red", |_, thing: String| Ok(thing.dark_red().to_string()))
    .add_function("dark_yellow", |_, thing: String| Ok(thing.dark_yellow().to_string()))
    
    // Styles
    .add_function("hidden", |_, thing: String| Ok(thing.hidden().to_string()))
    .add_function("reset", |_, thing: String| Ok(thing.reset().to_string()))
    .add_function("negative", |_, thing: String| Ok(thing.negative().to_string()))
    .add_function("slow_blink", |_, thing: String| Ok(thing.slow_blink().to_string()))
    .add_function("rapid_blink", |_, thing: String| Ok(thing.rapid_blink().to_string()))
    .add_function("italic", |_, thing: String| Ok(thing.italic().to_string()))
    .add_function("bold", |_, thing: String| Ok(thing.bold().to_string()))
    .add_function("underline", |_, thing: String| Ok(thing.underline(crossterm::style::Color::Grey).to_string()))
    .add_function("underline_white", |_, thing: String| Ok(thing.underline_white().to_string()))
    .add_function("underline_blue", |_, thing: String| Ok(thing.underline_blue().to_string()))
    .add_function("underline_black", |_, thing: String| Ok(thing.underline_black().to_string()))
    .add_function("underline_cyan", |_, thing: String| Ok(thing.underline_cyan().to_string()))
    .add_function("underline_green", |_, thing: String| Ok(thing.underline_green().to_string()))
    .add_function("underline_grey", |_, thing: String| Ok(thing.underline_grey().to_string()))
    .add_function("underline_magenta", |_, thing: String| Ok(thing.underline_magenta().to_string()))
    .add_function("underline_red", |_, thing: String| Ok(thing.underline_red().to_string()))
    .add_function("underline_yellow", |_, thing: String| Ok(thing.underline_yellow().to_string()))
    .add_function("underline_dark_cyan", |_, thing: String| Ok(thing.underline_dark_cyan().to_string()))
    .add_function("underline_dark_green", |_, thing: String| Ok(thing.underline_dark_green().to_string()))
    .add_function("underline_dark_grey", |_, thing: String| Ok(thing.underline_dark_grey().to_string()))
    .add_function("underline_dark_magenta", |_, thing: String| Ok(thing.underline_dark_magenta().to_string()))
    .add_function("underline_dark_red", |_, thing: String| Ok(thing.underline_dark_red().to_string()))
    .add_function("underline_dark_yellow", |_, thing: String| Ok(thing.underline_dark_yellow().to_string()))
    
    .add_function("on_white", |_, thing: String| Ok(thing.on_white().to_string()))
    .add_function("on_blue", |_, thing: String| Ok(thing.on_blue().to_string()))
    .add_function("on_black", |_, thing: String| Ok(thing.on_black().to_string()))
    .add_function("on_cyan", |_, thing: String| Ok(thing.on_cyan().to_string()))
    .add_function("on_green", |_, thing: String| Ok(thing.on_green().to_string()))
    .add_function("on_grey", |_, thing: String| Ok(thing.on_grey().to_string()))
    .add_function("on_magenta", |_, thing: String| Ok(thing.on_magenta().to_string()))
    .add_function("on_red", |_, thing: String| Ok(thing.on_red().to_string()))
    .add_function("on_yellow", |_, thing: String| Ok(thing.on_yellow().to_string()))
    .add_function("on_dark_cyan", |_, thing: String| Ok(thing.on_dark_cyan().to_string()))
    .add_function("on_dark_green", |_, thing: String| Ok(thing.on_dark_green().to_string()))
    .add_function("on_dark_grey", |_, thing: String| Ok(thing.on_dark_grey().to_string()))
    .add_function("on_dark_magenta", |_, thing: String| Ok(thing.on_dark_magenta().to_string()))
    .add_function("on_dark_red", |_, thing: String| Ok(thing.on_dark_red().to_string()))
    .add_function("on_dark_yellow", |_, thing: String| Ok(thing.on_dark_yellow().to_string()))

    .add_function("print", |_, args: mlua::Variadic<mlua::Value>| {
      stdout().queue(
        style::PrintStyledContent(args.into_iter().map(|x| match x {
          mlua::Value::String(s) => s.to_str().unwrap().to_string(),
          mlua::Value::UserData(s) => s.borrow_mut::<ConsoleString>().unwrap().string.to_string(),
          _ => "".to_string()
        }).collect::<Vec<String>>().join(" ").stylize())
      )?;
      Ok(())
    })
    
    // Size
    .add_function("size", |_, ()| Ok(crossterm::terminal::size()?))

    // Cursor
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

    // Events
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
    .add_file("console.lua", include_str!("../builtins/console.lua"))
    .into();
}
