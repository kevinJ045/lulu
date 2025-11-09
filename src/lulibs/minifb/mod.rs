use fontdue::Font;
use image::{DynamicImage, GenericImageView, RgbaImage, imageops::FilterType};
use minifb::{MouseButton, Window, WindowOptions};
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::{
  Arc, Mutex, OnceLock,
  atomic::{AtomicUsize, Ordering},
};
use std::thread;
use std::time::Duration;

use crate::ops::TOK_ASYNC_HANDLES;
use crate::{lulibs::bytes::LuluByteArray, ops::std::create_std_module};

pub static LOADED_IMAGES: OnceLock<Mutex<HashMap<String, DynamicImage>>> = OnceLock::new();
pub static RESIZED_CACHE: OnceLock<Mutex<HashMap<(String, u32, u32), RgbaImage>>> = OnceLock::new();

pub struct LuluMinifbWindow {
  width: Arc<AtomicUsize>,
  height: Arc<AtomicUsize>,
  title: String,
  font: Font,
  buffer: Arc<Mutex<Vec<u32>>>,
  callback: Arc<Mutex<Option<mlua::Function>>>,
  options: Option<mlua::Table>,
}

fn ensure_loaded_images() {
  LOADED_IMAGES.get_or_init(|| Mutex::new(HashMap::new()));
}

impl LuluMinifbWindow {
  pub fn new(title: String, width: usize, height: usize, options: Option<mlua::Table>) -> Self {
    let font_bytes = include_bytes!("../../../assets/DejaVuSansMono.ttf") as &[u8];
    let font =
      Font::from_bytes(font_bytes, fontdue::FontSettings::default()).expect("Failed to load font");

    Self {
      title,
      width: Arc::new(AtomicUsize::new(width)),
      height: Arc::new(AtomicUsize::new(height)),
      font,
      buffer: Arc::new(Mutex::new(vec![0; width * height])),
      callback: Arc::new(Mutex::new(None)),
      options,
    }
  }

  pub fn start(&mut self) {
    let title = self.title.clone();
    let options = self.options.clone();
    let width = Arc::clone(&self.width);
    let height = Arc::clone(&self.height);
    let buffer = Arc::clone(&self.buffer);
    let callback = Arc::clone(&self.callback);

    TOK_ASYNC_HANDLES
      .lock()
      .unwrap()
      .push(tokio::spawn(async move {
        let mut woptions = WindowOptions::default();

        if let Some(options) = options {
          if let Ok(resize) = options.get::<bool>("resize") {
            woptions.resize = resize;
          }

          if let Ok(borderless) = options.get::<bool>("borderless") {
            woptions.borderless = borderless;
          }

          if let Ok(transparency) = options.get::<bool>("transparency") {
            woptions.transparency = transparency;
          }

          if let Ok(topmost) = options.get::<bool>("topmost") {
            woptions.topmost = topmost;
          }

          if let Ok(none) = options.get::<bool>("none") {
            woptions.none = none;
          }
        }

        let mut window = Window::new(
          &title,
          width.load(Ordering::Relaxed),
          height.load(Ordering::Relaxed),
          woptions,
        )
        .expect("Failed to create window");

        while window.is_open() {
          let (w, h) = window.get_size();

          if w != width.load(Ordering::Relaxed) || h != height.load(Ordering::Relaxed) {
            width.store(w, Ordering::Relaxed);
            height.store(h, Ordering::Relaxed);

            if let Some(cb) = &*callback.lock().unwrap() {
              let _ = cb.call::<()>(("resize", w, h));
            }

            {
              let mut buf = buffer.lock().unwrap();
              buf.resize(w * h, 0);
            }
          }

          {
            let buf = buffer.lock().unwrap();
            if let Err(_) = window.update_with_buffer(&buf, w, h) {
              break;
            }
          }

          window.get_keys().iter().for_each(|key| {
            if let Some(cb) = &*callback.lock().unwrap() {
              let _ = cb.call::<()>(("key_down", format!("{:?}", key)));
            }
          });

          window
            .get_keys_pressed(minifb::KeyRepeat::No)
            .iter()
            .for_each(|key| {
              if let Some(cb) = &*callback.lock().unwrap() {
                let _ = cb.call::<()>(("key_pressed", format!("{:?}", key)));
              }
            });

          window.get_keys_released().iter().for_each(|key| {
            if let Some(cb) = &*callback.lock().unwrap() {
              let _ = cb.call::<()>(("key_up", format!("{:?}", key)));
            }
          });

          let (mx, my) = window
            .get_mouse_pos(minifb::MouseMode::Clamp)
            .unwrap_or((0.0, 0.0));
          if let Some(cb) = &*callback.lock().unwrap() {
            let _ = cb.call::<()>(("mouse_move", mx as usize, my as usize));
          }

          for button in [MouseButton::Left, MouseButton::Right, MouseButton::Middle] {
            if window.get_mouse_down(button) {
              if let Some(cb) = &*callback.lock().unwrap() {
                let _ = cb.call::<()>(("mouse_down", format!("{:?}", button)));
              }
            }
          }

          if let Some(cb) = &*callback.lock().unwrap() {
            let _ = cb.call::<()>("update");
          }

          thread::sleep(Duration::from_millis(16));
        }
      }));
  }

  fn put_pixel(&mut self, x: isize, y: isize, color: u32) {
    let width = self.width.load(Ordering::Relaxed);
    let height = self.height.load(Ordering::Relaxed);
    if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
      let mut buffer = self.buffer.lock().unwrap();
      buffer[y as usize * width + x as usize] = color;
    }
  }

  fn fill_rect_fast(&mut self, x: isize, y: isize, w: isize, h: isize, color: u32) {
    if w <= 0 || h <= 0 {
      return;
    }
    let mut buffer = self.buffer.lock().unwrap();
    let width = self.width.load(Ordering::Relaxed);
    let height = self.height.load(Ordering::Relaxed);
    let x = x.max(0) as usize;
    let y = y.max(0) as usize;
    let w = w as usize;
    let h = h as usize;

    for row in y..(y + h).min(height) {
      let start = row * width + x;
      let end = start + w.min(width - x);
      buffer[start..end].fill(color);
    }
  }

  fn draw_line_fast(&mut self, mut x0: isize, mut y0: isize, x1: isize, y1: isize, color: u32) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
      self.put_pixel(x0, y0, color);
      if x0 == x1 && y0 == y1 {
        break;
      }
      let e2 = 2 * err;
      if e2 >= dy {
        err += dy;
        x0 += sx;
      }
      if e2 <= dx {
        err += dx;
        y0 += sy;
      }
    }
  }

  fn draw_circle_fast(&mut self, cx: isize, cy: isize, r: isize, color: u32, fill: bool) {
    if r <= 0 {
      return;
    }
    let mut x = r;
    let mut y = 0;
    let mut err = 0;

    while x >= y {
      let points = [
        (cx + x, cy + y),
        (cx + y, cy + x),
        (cx - y, cy + x),
        (cx - x, cy + y),
        (cx - x, cy - y),
        (cx - y, cy - x),
        (cx + y, cy - x),
        (cx + x, cy - y),
      ];
      if fill {
        for yy in -y..=y {
          self.fill_rect_fast(cx - x, cy + yy, 2 * x + 1, 1, color);
          self.fill_rect_fast(cx - y, cy + yy, 2 * y + 1, 1, color);
        }
      } else {
        for &(px, py) in &points {
          self.put_pixel(px, py, color);
        }
      }
      y += 1;
      if err <= 0 {
        err += 2 * y + 1;
      }
      if err > 0 {
        x -= 1;
        err -= 2 * x + 1;
      }
    }
  }
}

impl LuaUserData for LuluMinifbWindow {
  fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
    fields.add_field_method_get("width", |_, this| Ok(this.width.load(Ordering::Relaxed)));

    fields.add_field_method_get("height", |_, this| Ok(this.height.load(Ordering::Relaxed)));
  }

  fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
    methods.add_method_mut("clear", |_, this, color: u32| {
      let mut buffer = this.buffer.lock().unwrap();
      buffer.fill(color);
      Ok(())
    });

    methods.add_method_mut("handle", |_, this, f: mlua::Function| {
      let mut callback = this.callback.lock().unwrap();
      *callback = Some(f);
      Ok(())
    });

    methods.add_method_mut(
      "draw_pixel",
      |_, this, (x, y, color): (isize, isize, u32)| {
        this.put_pixel(x, y, color);
        Ok(())
      },
    );

    methods.add_method_mut(
      "draw_line",
      |_, this, (x0, y0, x1, y1, color): (isize, isize, isize, isize, u32)| {
        this.draw_line_fast(x0, y0, x1, y1, color);
        Ok(())
      },
    );

    methods.add_method_mut(
      "draw_rect",
      |_, this, (x, y, w, h, color): (isize, isize, isize, isize, u32)| {
        // unfilled rect: draw four lines
        this.draw_line_fast(x, y, x + w - 1, y, color);
        this.draw_line_fast(x, y, x, y + h - 1, color);
        this.draw_line_fast(x + w - 1, y, x + w - 1, y + h - 1, color);
        this.draw_line_fast(x, y + h - 1, x + w - 1, y + h - 1, color);
        Ok(())
      },
    );

    methods.add_method_mut(
      "fill_rect",
      |_, this, (x, y, w, h, color): (isize, isize, isize, isize, u32)| {
        this.fill_rect_fast(x, y, w, h, color);
        Ok(())
      },
    );

    methods.add_method_mut(
      "draw_circle",
      |_, this, (cx, cy, r, color): (isize, isize, isize, u32)| {
        this.draw_circle_fast(cx, cy, r, color, false);
        Ok(())
      },
    );

    methods.add_method_mut(
      "fill_circle",
      |_, this, (cx, cy, r, color): (isize, isize, isize, u32)| {
        this.draw_circle_fast(cx, cy, r, color, true);
        Ok(())
      },
    );

    methods.add_method_mut("load_image", |_, _this, img: mlua::Value| {
      ensure_loaded_images();

      let mut cache = LOADED_IMAGES.get().unwrap().lock().unwrap();

      match img {
        mlua::Value::String(s) => {
          let s = s.to_str()?;
          let key = s.to_string();

          if cache.contains_key(&key) {
            return Ok(key);
          }

          if let Ok(bytes) = std::fs::read(s.to_string()) {
            if let Ok(img) = image::load_from_memory(&bytes) {
              cache.insert(key.clone(), img);
              return Ok(key);
            } else {
              return Err(mlua::Error::external("Failed to decode image from file"));
            }
          }

          Err(mlua::Error::external("Invalid file path or URL"))
        }

        mlua::Value::UserData(ud) => {
          if let Ok(bytes) = ud.borrow::<LuluByteArray>() {
            let key = format!("mem_{:p}", &*bytes.bytes);
            if cache.contains_key(&key) {
              return Ok(key);
            }

            if let Ok(img) = image::load_from_memory(&bytes.bytes) {
              cache.insert(key.clone(), img);
              Ok(key)
            } else {
              Err(mlua::Error::external("Failed to decode image from memory"))
            }
          } else {
            Err(mlua::Error::external("Expected ByteArray"))
          }
        }

        _ => Err(mlua::Error::external("Expected string (path) or ByteArray")),
      }
    });

    methods.add_method_mut(
    "draw_image",
    |_, this, (dst_x, dst_y, image_src, dst_w, dst_h): (u32, u32, String, Option<u32>, Option<u32>)| {
        ensure_loaded_images();
        let cache = LOADED_IMAGES.get().unwrap().lock().unwrap();
        let img = match cache.get(&image_src) {
            Some(img) => img,
            None => return Ok(()),
        };

        // Resize if requested
        let img_ref = if let (Some(dw), Some(dh)) = (dst_w, dst_h) {
            Box::new(img.resize(dw, dh, FilterType::Triangle)) as Box<DynamicImage>
        } else {
            Box::new(img.clone())
        };

        let rgba = img_ref.to_rgba8();
        let (iw, ih) = rgba.dimensions();

        let width = this.width.load(std::sync::atomic::Ordering::Relaxed) as usize;
        let height = this.height.load(std::sync::atomic::Ordering::Relaxed) as usize;
        let mut buffer = this.buffer.lock().unwrap();

        let start_y = dst_y.min(height as u32) as usize;
        let start_x = dst_x.min(width as u32) as usize;
        let copy_w = ((dst_x + iw) as usize).min(width) - start_x;
        let copy_h = ((dst_y + ih) as usize).min(height) - start_y;

        for y in 0..copy_h {
    let dst_row_start = (start_y + y) * width + start_x;

    for x in 0..copy_w {
        let dst_idx = dst_row_start + x;
        let px = rgba.get_pixel(x as u32, y as u32);
        let r = px[0] as f32;
        let g = px[1] as f32;
        let b = px[2] as f32;
        let a = px[3] as f32 / 255.0;

        if a == 0.0 {
            continue;
        } else if (a - 1.0).abs() < std::f32::EPSILON {
            buffer[dst_idx] = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        } else {
            let dst_pixel = buffer[dst_idx];
            let dr = ((dst_pixel >> 16) & 0xFF) as f32;
            let dg = ((dst_pixel >> 8) & 0xFF) as f32;
            let db = (dst_pixel & 0xFF) as f32;

            let nr = (r * a + dr * (1.0 - a)).round().clamp(0.0, 255.0) as u32;
            let ng = (g * a + dg * (1.0 - a)).round().clamp(0.0, 255.0) as u32;
            let nb = (b * a + db * (1.0 - a)).round().clamp(0.0, 255.0) as u32;

            buffer[dst_idx] = (nr << 16) | (ng << 8) | nb;
        }
    }
}

        Ok(())
    },
);

    methods.add_method_mut(
      "draw_text",
      |_, this, (x, y, size, text, color): (isize, isize, f32, String, u32)| {
        let mut cursor_x = x as f32;
        let mut buffer = this.buffer.lock().unwrap();

        let width = this.width.load(Ordering::Relaxed);
        let height = this.height.load(Ordering::Relaxed);

        for c in text.chars() {
          let (metrics, bitmap) = this.font.rasterize(c, size);
          for row in 0..metrics.height {
            let y_pos = y as isize + row as isize;
            if y_pos < 0 || y_pos as usize >= height {
              continue;
            }
            for col in 0..metrics.width {
              let x_pos = cursor_x as isize + col as isize;
              if x_pos < 0 || x_pos as usize >= width {
                continue;
              }
              let alpha = bitmap[row * metrics.width + col];
              if alpha > 0 {
                let idx = y_pos as usize * width + x_pos as usize;
                buffer[idx] = color;
              }
            }
          }
          cursor_x += metrics.advance_width;
        }
        Ok(())
      },
    );

    methods.add_method_mut("start", |_, this, ()| {
      this.start();
      Ok(())
    });
  }
}

// === Module registration ===

pub fn into_module() {
  create_std_module("minifb")
    .add_function(
      "window",
      |_, (title, width, height, options): (String, usize, usize, Option<mlua::Table>)| {
        Ok(LuluMinifbWindow::new(title, width, height, options))
      },
    )
    .on_register(|_, minifb_mod| Ok(minifb_mod))
    .into();
}
