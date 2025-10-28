use emmylua_code_analysis::{EmmyLuaAnalysis};
use lulu::conf::{find_lulu_conf, load_lulu_conf};
use lulu::lulu::STD_FILE;
use lulu::sourcemap::{generate_sourcemap, lookup_lua_to_lulu};
use std::panic;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use lulu::compiler::Compiler;

struct Backend {
  client: Client,
  analysis: Arc<RwLock<EmmyLuaAnalysis>>,
}

impl Backend {
  pub fn new(client: Client) -> Self {

    let mut analysis = EmmyLuaAnalysis::new();
    
    let std_code = include_str!("./builtins/stddef.lua");

    let tempdir = tempfile::tempdir().unwrap();
    let std_path = tempdir.path().join("stdlib.lua");
    std::fs::write(std_path.clone(), std_code).unwrap();

    analysis.init_std_lib(None);

    analysis.add_library_workspace(tempdir.path().to_path_buf());
    analysis.update_file_by_path(&std_path, Some(std_code.to_string()));    
    
    Self {
      analysis: Arc::new(RwLock::new(analysis)),
      client
    }
  }

  async fn on_change(&self, uri: Url, text: &str) {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    let compiler_result = panic::catch_unwind(|| {
      let mut compiler = Compiler::new(None);
      let mut conf = None;
      if let Some(conpath) = find_lulu_conf(std::path::PathBuf::from(uri.path())) {
        conf = match load_lulu_conf(&mlua::Lua::new(), conpath) {
          Ok(c) => {
            if let Some(macros) = c.macros.clone() {
              compiler.compile(&macros, None, None);
            }
            Some(c)
          }
          _ => None,
        }
      }
      compiler.compile(STD_FILE, None, None);
      (compiler.compile(text, Some(uri.path().to_string()), conf.clone()), conf)
    });

    if let Err(panic_payload) = compiler_result {
      let message = if let Some(s) = panic_payload.downcast_ref::<&'static str>() {
        s.to_string()
      } else if let Some(s) = panic_payload.downcast_ref::<String>() {
        s.clone()
      } else {
        "Compiler panicked".to_string()
      };
      diagnostics.push(Diagnostic::new_simple(
        Range::new(Position::new(0, 0), Position::new(0, 1)),
        message,
      ));
    } else if let Ok((compiled_text, _conf)) = compiler_result {
      let mut analysis = self.analysis.write().await;


      let f = analysis
        .update_file_by_path(&std::path::PathBuf::from(uri.path()), Some(compiled_text.to_string()))
        .unwrap();

      let d = analysis.diagnose_file(f, CancellationToken::new());

      let srcmap = generate_sourcemap(text, &compiled_text);

      if let Some(diags) = d {
        for diag in diags {
          let (start_line, start_col) = lookup_lua_to_lulu(
            diag.range.start.line as usize,
            diag.range.start.character as usize,
            &srcmap,
          )
          .unwrap_or((
            diag.range.start.line as usize,
            diag.range.start.character as usize,
          ));
          let (end_line, end_col) = lookup_lua_to_lulu(
            diag.range.start.line as usize,
            diag.range.start.character as usize,
            &srcmap,
          )
          .unwrap_or((
            diag.range.start.line as usize,
            diag.range.start.character as usize,
          ));

          let start = Position::new(start_line as u32, start_col as u32);
          let end = Position::new(end_line as u32, end_col as u32);

          if diag.message == "self maybe nil" {
            continue
          }

          let severity: DiagnosticSeverity = match format!("{:?}", diag.severity).to_lowercase().as_str() {
            "some(error)" => DiagnosticSeverity::ERROR,
            "some(hint)" => DiagnosticSeverity::HINT,
            "some(warning)" => DiagnosticSeverity::WARNING,
            _ => DiagnosticSeverity::INFORMATION,
          };

          diagnostics.push(Diagnostic {
            range: Range::new(start, end),
            severity: Some(severity),
            code: None,
            code_description: None,
            source: Some("emmylua".to_string()),
            message: diag.message.clone(),
            related_information: None,
            tags: None,
            data: diag.data.clone(),
          });
        }
      }
    }

    self
      .client
      .publish_diagnostics(uri, diagnostics, None)
      .await;
  }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
  async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
    Ok(InitializeResult {
      server_info: Some(ServerInfo {
        name: "lulu-lsp".to_string(),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
      }),
      capabilities: ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        ..Default::default()
      },
    })
  }

  async fn initialized(&self, _: InitializedParams) {
    self
      .client
      .log_message(
        MessageType::INFO,
        "lulu-lsp initialized with emmylua_check diagnostics.",
      )
      .await;
  }

  async fn shutdown(&self) -> Result<()> {
    Ok(())
  }

  async fn did_change(&self, params: DidChangeTextDocumentParams) {
    if let Some(change) = params.content_changes.into_iter().next() {
      self.on_change(params.text_document.uri, &change.text).await;
    }
  }
}

#[tokio::main]
async fn main() {
  let stdin = tokio::io::stdin();
  let stdout = tokio::io::stdout();

  let (service, socket) = LspService::new(|client| Backend::new(client));
  Server::new(stdin, stdout, socket).serve(service).await;
}
