use crossterm::{
  event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use mlua::prelude::*;
use std::io;
use tui::{
  Frame, Terminal,
  backend::{Backend, CrosstermBackend},
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  symbols,
  text::{Span, Spans},
  widgets::*,
};

use crate::ops::std::create_std_module;

fn table_to_vec_pairs_f64(t: LuaTable) -> LuaResult<Vec<(f64, f64)>> {
  let mut out = Vec::new();
  for pair in t.sequence_values::<LuaValue>() {
    let v = pair?;
    match v {
      LuaValue::Table(tbl) => {
        let first = tbl.get::<LuaValue>(1)?;
        let second = tbl.get::<LuaValue>(2)?;
        let x = match first {
          LuaValue::Number(n) => n,
          _ => {
            return Err(LuaError::FromLuaConversionError {
              from: "value",
              to: "f64".to_string(),
              message: Some("expected number".into()),
            });
          }
        };
        let y = match second {
          LuaValue::Number(n) => n,
          _ => {
            return Err(LuaError::FromLuaConversionError {
              from: "value",
              to: "f64".to_string(),
              message: Some("expected number".into()),
            });
          }
        };
        out.push((x as f64, y as f64));
      }
      _ => {
        return Err(LuaError::FromLuaConversionError {
          from: "value",
          to: "table".to_string(),
          message: Some("expected table pair".into()),
        });
      }
    }
  }
  Ok(out)
}

fn table_to_vec_pairs_string_u64(t: LuaTable) -> LuaResult<Vec<(String, u64)>> {
  let mut out = Vec::new();
  for pair in t.sequence_values::<LuaValue>() {
    let v = pair?;
    match v {
      LuaValue::Table(tbl) => {
        let first = tbl.get::<LuaValue>(1)?;
        let second = tbl.get::<LuaValue>(2)?;
        let s = match first {
          LuaValue::String(sv) => sv.to_str()?.to_string(),
          LuaValue::Number(n) => n.to_string(),
          LuaValue::Integer(i) => i.to_string(),
          _ => {
            return Err(LuaError::FromLuaConversionError {
              from: "value",
              to: "String".to_string(),
              message: Some("expected string/number".into()),
            });
          }
        };
        let vnum = match second {
          LuaValue::Integer(i) => i as u64,
          LuaValue::Number(n) => n as u64,
          _ => {
            return Err(LuaError::FromLuaConversionError {
              from: "value",
              to: "u64".to_string(),
              message: Some("expected integer/number".into()),
            });
          }
        };
        out.push((s, vnum));
      }
      _ => {
        return Err(LuaError::FromLuaConversionError {
          from: "value",
          to: "table".to_string(),
          message: Some("expected table pair".into()),
        });
      }
    }
  }
  Ok(out)
}

pub struct TuiApp {
  terminal: Terminal<CrosstermBackend<io::Stdout>>,
  open: bool
}

impl TuiApp {
  fn new() -> LuaResult<Self> {
    enable_raw_mode().map_err(LuaError::external)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(LuaError::external)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).map_err(LuaError::external)?;
    Ok(Self { terminal, open: true })
  }

  pub fn draw(&mut self, layout: LuaAnyUserData) -> LuaResult<()> {
    let layout = layout.borrow::<LuaLayout>()?;
    self
      .terminal
      .draw(|f| layout.render(f, f.size()))
      .map_err(LuaError::external)?;

    Ok(())
  }

  pub fn close(&mut self) -> LuaResult<()>{
    self.open = false;
    Ok(())
  }

  pub fn poll(&mut self, lua: &Lua, timeout_ms: Option<u64>) -> LuaResult<Option<LuaTable>> {
    let timeout = std::time::Duration::from_millis(timeout_ms.unwrap_or(0));

    if event::poll(timeout).map_err(LuaError::external)? {
      match event::read().map_err(LuaError::external)? {
        Event::Key(KeyEvent { code, .. }) => {
          let tbl = lua.create_table()?;

          match code {
            KeyCode::Char(c) => {
              tbl.set("type", "key")?;
              tbl.set("key", c.to_string())?;
            }
            KeyCode::Enter => {
              tbl.set("type", "key")?;
              tbl.set("key", "enter")?;
            }
            KeyCode::Esc => {
              tbl.set("type", "key")?;
              tbl.set("key", "esc")?;
            }
            KeyCode::Left => {
              tbl.set("type", "key")?;
              tbl.set("key", "left")?;
            }
            KeyCode::Right => {
              tbl.set("type", "key")?;
              tbl.set("key", "right")?;
            }
            KeyCode::Up => {
              tbl.set("type", "key")?;
              tbl.set("key", "up")?;
            }
            KeyCode::Down => {
              tbl.set("type", "key")?;
              tbl.set("key", "down")?;
            }
            _ => {
              tbl.set("type", "key")?;
              tbl.set("key", "unknown")?;
            }
          }

          return Ok(Some(tbl));
        }
        Event::Mouse(evt) => {
          let tbl = lua.create_table()?;
          tbl.set("type", "mouse")?;
          tbl.set("button", format!("{:?}", evt.kind))?;
          tbl.set("column", evt.column)?;
          tbl.set("row", evt.row)?;
          tbl.set("modifier", format!("{:?}", evt.modifiers))?;
          return Ok(Some(tbl));
        },
        Event::FocusGained => {
          let tbl = lua.create_table()?;
          tbl.set("type", "focus")?;
          tbl.set("focus", true)?;
          return Ok(Some(tbl));
        },
        Event::FocusLost => {
          let tbl = lua.create_table()?;
          tbl.set("type", "focus")?;
          tbl.set("focus", false)?;
          return Ok(Some(tbl));
        },
        Event::Paste(data) => {
          let tbl = lua.create_table()?;
          tbl.set("type", "resize")?;
          tbl.set("data", data)?;
          return Ok(Some(tbl));
        },
        Event::Resize(w, h) => {
          let tbl = lua.create_table()?;
          tbl.set("type", "resize")?;
          tbl.set("width", w)?;
          tbl.set("height", h)?;
          return Ok(Some(tbl));
        }
      }
    }

    Ok(None)
  }
}

impl Drop for TuiApp {
  fn drop(&mut self) {
    let _ = disable_raw_mode();
    let _ = execute!(
      self.terminal.backend_mut(),
      LeaveAlternateScreen,
      DisableMouseCapture
    );
  }
}

impl LuaUserData for TuiApp {
  fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
    methods.add_method_mut("draw", |_, this, layout: LuaAnyUserData| this.draw(layout));
    methods.add_method_mut("poll", |lua, this, t: Option<u64>| this.poll(lua, t));
    methods.add_method_mut("is_open", |_, this, ()| Ok(this.open));
    methods.add_method_mut("close", |_, this, ()| this.close());
  }
}

#[derive(Clone)]
pub enum LuaWidget {
  Paragraph(LuaParagraph),
  Gauge(LuaGauge),
  Block(LuaBlock),
  List(LuaList),
  Tabs(LuaTabs),
  Table(LuaTableWidget),
  BarChart(LuaBarChart),
  Chart(LuaChart),
  Layout(Box<LuaLayout>),
}

impl LuaWidget {
  fn render<B: tui::backend::Backend>(&self, f: &mut tui::Frame<B>, area: Rect) {
    match self {
      LuaWidget::Paragraph(p) => f.render_widget(p.to_tui(), area),
      LuaWidget::Gauge(g) => f.render_widget(g.to_tui(), area),
      LuaWidget::Block(b) => f.render_widget(b.to_tui(), area),
      LuaWidget::List(l) => f.render_widget(l.to_tui(), area),
      LuaWidget::Tabs(t) => f.render_widget(t.to_tui(), area),
      LuaWidget::Table(t) => f.render_widget(t.to_tui(), area),
      LuaWidget::BarChart(b) => b.render(f, area),
      LuaWidget::Chart(c) => f.render_widget(c.to_tui(), area),
      LuaWidget::Layout(l) => l.render(f, area),
    }
  }
}

#[derive(Clone)]
pub struct LuaLayout {
  direction: Direction,
  constraints: Vec<Constraint>,
  children: Vec<LuaWidget>,
}

impl LuaLayout {
  fn new(cfg: LuaTable) -> LuaResult<Self> {
    let dir_str: Option<String> = cfg.get("direction")?;
    let direction = match dir_str.as_deref() {
      Some("horizontal") => Direction::Horizontal,
      _ => Direction::Vertical,
    };

    let mut constraints = Vec::new();
    if let Ok(list) = cfg.get::<LuaTable>("constraints") {
      for pair in list.sequence_values::<String>() {
        let val = pair?;
        if val.ends_with('%') {
          let num: f64 = val[..val.len() - 1].parse().unwrap_or(100.0);
          constraints.push(Constraint::Percentage(num as u16));
        } else {
          constraints.push(Constraint::Length(val.parse().unwrap_or(1)));
        }
      }
    }

    let mut children = Vec::new();
    if let Ok(tbl) = cfg.get::<LuaTable>("children") {
      for pair in tbl.sequence_values::<LuaAnyUserData>() {
        let ud = pair?;
        if let Ok(l) = ud.borrow::<LuaLayout>() {
          children.push(LuaWidget::Layout(Box::new(l.clone())));
        } else if let Ok(p) = ud.borrow::<LuaParagraph>() {
          children.push(LuaWidget::Paragraph(p.clone()));
        } else if let Ok(g) = ud.borrow::<LuaGauge>() {
          children.push(LuaWidget::Gauge(g.clone()));
        } else if let Ok(b) = ud.borrow::<LuaBlock>() {
          children.push(LuaWidget::Block(b.clone()));
        } else if let Ok(l) = ud.borrow::<LuaList>() {
          children.push(LuaWidget::List(l.clone()));
        } else if let Ok(t) = ud.borrow::<LuaTabs>() {
          children.push(LuaWidget::Tabs(t.clone()));
        } else if let Ok(t) = ud.borrow::<LuaTableWidget>() {
          children.push(LuaWidget::Table(t.clone()));
        } else if let Ok(b) = ud.borrow::<LuaBarChart>() {
          children.push(LuaWidget::BarChart(b.clone()));
        } else if let Ok(c) = ud.borrow::<LuaChart>() {
          children.push(LuaWidget::Chart(c.clone()));
        }
      }
    }

    Ok(Self {
      direction,
      constraints,
      children,
    })
  }

  fn render<B: tui::backend::Backend>(&self, f: &mut tui::Frame<B>, area: Rect) {
    let areas = Layout::default()
      .direction(self.direction.clone())
      .constraints(self.constraints.clone())
      .split(area);
    for (i, child) in self.children.iter().enumerate() {
      if let Some(area) = areas.get(i) {
        child.render(f, *area);
      }
    }
  }
}

impl LuaUserData for LuaLayout {}

#[derive(Clone)]
pub struct LuaParagraph {
  text: String,
}
impl LuaParagraph {
  fn new(text: String) -> Self {
    Self { text }
  }
  fn to_tui(&self) -> Paragraph<'_> {
    Paragraph::new(self.text.as_str()).block(Block::default().borders(Borders::ALL))
  }
}
impl LuaUserData for LuaParagraph {}

#[derive(Clone)]
pub struct LuaGauge {
  ratio: f64,
}
impl LuaGauge {
  fn new(ratio: f64) -> Self {
    Self { ratio }
  }
  fn to_tui(&self) -> Gauge<'_> {
    Gauge::default()
      .block(Block::default().borders(Borders::ALL))
      .gauge_style(Style::default().fg(Color::Yellow))
      .ratio(self.ratio)
  }
}
impl LuaUserData for LuaGauge {}

#[derive(Clone)]
pub struct LuaBlock {
  title: Option<String>,
}
impl LuaBlock {
  fn new(title: Option<String>) -> Self {
    Self { title }
  }
  fn to_tui(&self) -> Block<'_> {
    let mut b = Block::default().borders(Borders::ALL);
    if let Some(t) = &self.title {
      b = b.title(t.as_str());
    }
    b
  }
}
impl LuaUserData for LuaBlock {}

#[derive(Clone)]
pub struct LuaList {
  items: Vec<String>,
}
impl LuaList {
  fn new(items: Vec<String>) -> Self {
    Self { items }
  }
  fn to_tui(&self) -> List<'_> {
    let items: Vec<ListItem> = self
      .items
      .iter()
      .map(|i| ListItem::new(Span::raw(i)))
      .collect();
    List::new(items)
      .block(Block::default().borders(Borders::ALL).title("List"))
      .highlight_style(Style::default().add_modifier(Modifier::BOLD))
  }
}
impl LuaUserData for LuaList {}

#[derive(Clone)]
pub struct LuaTabs {
  titles: Vec<String>,
  index: usize,
}
impl LuaTabs {
  fn new(titles: Vec<String>, index: usize) -> Self {
    Self { titles, index }
  }
  fn to_tui(&self) -> Tabs<'_> {
    let titles: Vec<Spans> = self.titles.iter().map(|t| Spans::from(t.clone())).collect();
    Tabs::new(titles)
      .block(Block::default().borders(Borders::ALL))
      .select(self.index)
      .highlight_style(Style::default().fg(Color::Yellow))
  }
}
impl LuaUserData for LuaTabs {}

#[derive(Clone)]
pub struct LuaTableWidget {
  headers: Vec<String>,
  rows: Vec<Vec<String>>,
}
impl LuaTableWidget {
  fn new(headers: Vec<String>, rows: Vec<Vec<String>>) -> Self {
    Self { headers, rows }
  }
  fn to_tui(&self) -> Table<'_> {
    let header = Row::new(self.headers.iter().map(|h| Cell::from(h.as_str())));
    let rows = self
      .rows
      .iter()
      .map(|r| Row::new(r.iter().map(|c| Cell::from(c.as_str()))));
    Table::new(rows)
      .header(header)
      .block(Block::default().borders(Borders::ALL).title("Table"))
      .widths(&[
        Constraint::Percentage(33),
        Constraint::Percentage(33),
        Constraint::Percentage(34),
      ])
  }
}
impl LuaUserData for LuaTableWidget {}

#[derive(Clone)]
pub struct LuaBarChart {
  data: Vec<(String, u64)>,
}
impl LuaBarChart {
  fn new(data: Vec<(String, u64)>) -> Self {
    Self { data }
  }
  fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
    let data: Vec<(&str, u64)> = self.data.iter().map(|(l, v)| (l.as_str(), *v)).collect();
    let chart = BarChart::default()
      .block(Block::default().borders(Borders::ALL).title("Bar Chart"))
      .data(&data)
      .bar_width(6)
      .bar_gap(2)
      .bar_style(Style::default().fg(Color::Cyan));

    f.render_widget(chart, area);
  }
}
impl LuaUserData for LuaBarChart {}

#[derive(Clone)]
pub struct LuaChart {
  points: Vec<(f64, f64)>,
}
impl LuaChart {
  fn new(points: Vec<(f64, f64)>) -> Self {
    Self { points }
  }
  fn to_tui(&self) -> Chart<'_> {
    let dataset = Dataset::default()
      .name("Line")
      .marker(symbols::Marker::Dot)
      .style(Style::default().fg(Color::Green))
      .data(&self.points);
    Chart::new(vec![dataset]).block(Block::default().borders(Borders::ALL).title("Chart"))
  }
}
impl LuaUserData for LuaChart {}

pub fn into_module() {
  create_std_module("tui")
    .add_function("app", |_, ()| TuiApp::new())
    .add_function("layout", |_, cfg: LuaTable| LuaLayout::new(cfg))
    .add_function("paragraph", |_, text: String| Ok(LuaParagraph::new(text)))
    .add_function("gauge", |_, ratio: f64| Ok(LuaGauge::new(ratio)))
    .add_function("block", |_, title: Option<String>| Ok(LuaBlock::new(title)))
    .add_function("list", |_, items: Vec<String>| Ok(LuaList::new(items)))
    .add_function("tabs", |_, (titles, idx): (Vec<String>, usize)| {
      Ok(LuaTabs::new(titles, idx))
    })
    .add_function(
      "table",
      |_, (headers, rows): (Vec<String>, Vec<Vec<String>>)| Ok(LuaTableWidget::new(headers, rows)),
    )
    .add_function("barchart", |_, data: LuaTable| {
      Ok(LuaBarChart::new(table_to_vec_pairs_string_u64(data)?))
    })
    .add_function("chart", |_, points: LuaTable| {
      Ok(LuaChart::new(table_to_vec_pairs_f64(points)?))
    })
    .into();
}
