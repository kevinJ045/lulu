use eframe::egui;
use mlua::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender};

// --- Data Model ---

#[derive(Clone)]
pub struct Widget {
  pub events: HashMap<String, LuaFunction>,
}

#[derive(Clone)]
pub struct Node {
  pub id: u64,
  pub widget: Option<Widget>,
  pub children: Vec<u64>,
  pub kind: NodeKind,
}

#[derive(Clone)]
pub enum NodeKind {
  Window { title: String },
  Button { label: String },
  Label { text: String },
}

#[derive(Default)]
struct UiRegistry {
  nodes: HashMap<u64, Node>,
  roots: Vec<u64>,
}

// --- Commands ---

#[derive(Debug)]
pub enum UiCommand {
  CreateWindow {
    id: u64,
    title: String,
  },
  CreateButton {
    id: u64,
    parent_id: u64,
    label: String,
  },
  CreateLabel {
    id: u64,
    parent_id: u64,
    text: String,
  },
  SetText {
    id: u64,
    text: String,
  },
  RegisterCallback {
    id: u64,
    event: String,
    func: LuaFunction,
  },
}

// --- Lua API Context ---

#[derive(Clone)]
pub struct LuaUiContext {
  pub command_sender: Sender<UiCommand>,
  pub id_generator: Arc<AtomicU64>,
  pub should_run: Arc<AtomicBool>,
}

impl LuaUserData for LuaUiContext {
  fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
    methods.add_method("window", |_, this, title: String| {
      let id = this.id_generator.fetch_add(1, Ordering::Relaxed);
      this
        .command_sender
        .send(UiCommand::CreateWindow { id, title })
        .unwrap();
      Ok(id)
    });

    methods.add_method("button", |_, this, (parent_id, label): (u64, String)| {
      let id = this.id_generator.fetch_add(1, Ordering::Relaxed);
      this
        .command_sender
        .send(UiCommand::CreateButton {
          id,
          parent_id,
          label,
        })
        .unwrap();
      Ok(id)
    });

    methods.add_method("label", |_, this, (parent_id, text): (u64, String)| {
      let id = this.id_generator.fetch_add(1, Ordering::Relaxed);
      this
        .command_sender
        .send(UiCommand::CreateLabel {
          id,
          parent_id,
          text,
        })
        .unwrap();
      Ok(id)
    });

    methods.add_method("set_text", |_, this, (id, text): (u64, String)| {
      this
        .command_sender
        .send(UiCommand::SetText { id, text })
        .unwrap();
      Ok(())
    });

    methods.add_method(
      "on",
      |_, this, (id, event, func): (u64, String, LuaFunction)| {
        // this
        //   .command_sender
        //   .send(UiCommand::RegisterCallback {
        //     id,
        //     event,
        //     func: func.into_static(),
        //   })
        //   .unwrap();
        Ok(())
      },
    );

    methods.add_method("run", |_, this, ()| {
      this.should_run.store(true, Ordering::Relaxed);
      Ok(())
    });
  }
}

// --- eframe App ---

struct LuluUIApp {
  lua: Lua,
  registry: UiRegistry,
  command_receiver: Receiver<UiCommand>,
  lua_ui_context: LuaUiContext,
}

impl LuluUIApp {
  fn new(lua: &Lua, command_receiver: Receiver<UiCommand>, lua_ui_context: LuaUiContext) -> Self {
    Self {
      lua,
      registry: UiRegistry::default(),
      command_receiver,
      lua_ui_context,
    }
  }

  fn process_commands(&mut self) {
    for cmd in self.command_receiver.try_iter() {
      match cmd {
        UiCommand::CreateWindow { id, title } => {
          let node = Node {
            id,
            widget: None,
            children: vec![],
            kind: NodeKind::Window { title },
          };
          self.registry.nodes.insert(id, node);
          self.registry.roots.push(id);
        }
        UiCommand::CreateButton {
          id,
          parent_id,
          label,
        } => {
          let widget = Widget {
            events: HashMap::new(),
          };
          let node = Node {
            id,
            widget: Some(widget),
            children: vec![],
            kind: NodeKind::Button { label },
          };
          self.registry.nodes.insert(id, node);
          if let Some(parent) = self.registry.nodes.get_mut(&parent_id) {
            parent.children.push(id);
          }
        }
        UiCommand::CreateLabel {
          id,
          parent_id,
          text,
        } => {
          let node = Node {
            id,
            widget: None,
            children: vec![],
            kind: NodeKind::Label { text },
          };
          self.registry.nodes.insert(id, node);
          if let Some(parent) = self.registry.nodes.get_mut(&parent_id) {
            parent.children.push(id);
          }
        }
        UiCommand::SetText { id, text } => {
          if let Some(node) = self.registry.nodes.get_mut(&id) {
            match &mut node.kind {
              NodeKind::Button { label } => *label = text,
              NodeKind::Label { text: lbl } => *lbl = text,
              _ => {}
            }
          }
        }
        UiCommand::RegisterCallback { id, event, func } => {
          if let Some(node) = self.registry.nodes.get_mut(&id) {
            if let Some(widget) = &mut node.widget {
              widget.events.insert(event, func);
            }
          }
        }
      }
    }
  }

  fn render_node(&self, ui: &mut egui::Ui, node_id: u64) {
    if let Some(node) = self.registry.nodes.get(&node_id).cloned() {
      match &node.kind {
        NodeKind::Button { label } => {
          if ui.button(label).clicked() {
            if let Some(widget) = &node.widget {
              if let Some(cb) = widget.events.get("click") {
                // Use the app's Lua context to call the function
                if let Err(e) = self.lua.scope(|scope| {
                  // let cb = cb.bind_to(scope)?;
                  // cb.call::<_, ()>(self.lua_ui_context.clone())
                  Ok(())
                }) {
                  eprintln!("Error in UI callback: {}", e);
                }
              }
            }
          }
        }
        NodeKind::Label { text } => {
          ui.label(text);
        }
        NodeKind::Window { .. } => { /* Handled separately */ }
      }

      for child_id in &node.children {
        self.render_node(ui, *child_id);
      }
    }
  }
}

impl eframe::App for LuluUIApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    self.process_commands();

    let roots = self.registry.roots.clone();
    for root_id in roots {
      if let Some(node) = self.registry.nodes.get(&root_id).cloned() {
        if let NodeKind::Window { title } = &node.kind {
          egui::Window::new(title).show(ctx, |ui| {
            for child_id in &node.children {
              self.render_node(ui, *child_id);
            }
          });
        }
      }
    }
  }
}

/// Entry point for running the UI.
/// This takes ownership of the Lua context and will run the UI loop on the main thread.
pub fn run(lua: &Lua, command_receiver: Receiver<UiCommand>, lua_ui_context: LuaUiContext) {
  let native_options = eframe::NativeOptions::default();
  eframe::run_native(
    "Lulu App",
    native_options,
    Box::new(|_cc| Box::new(LuluUIApp::new(lua, command_receiver, lua_ui_context))),
  );
}
