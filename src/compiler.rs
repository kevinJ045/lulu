use std::collections::HashMap;

use crate::conf::LuluConf;

#[derive(Debug, Clone)]
pub struct MacroDefinition {
  pub name: String,
  pub params: Vec<String>,
  pub body: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct MacroRegistry {
  macros: HashMap<String, MacroDefinition>,
}

impl MacroRegistry {
  pub fn new() -> Self {
    let mut macros = HashMap::new();

    macros.insert(
      "for_each".to_string(),
      MacroDefinition {
        name: "for_each".to_string(),
        params: vec![
          "item".to_string(),
          "iterator".to_string(),
          "block".to_string(),
        ],
        body: tokenize("for $item in ipairs($iterator) do\n$block\nend"),
      },
    );
    macros.insert(
      "match".to_string(),
      MacroDefinition {
        name: "for_each".to_string(),
        params: vec!["item".to_string(), "iterator".to_string()],
        body: tokenize("into(nil)"),
      },
    );
    macros.insert(
      "lml".to_string(),
      MacroDefinition {
        name: "lml".to_string(),
        params: vec!["expr".to_string()],
        body: tokenize("lml_into(nil)"),
      },
    );
    macros.insert(
      "cfg".to_string(),
      MacroDefinition {
        name: "cfg".to_string(),
        params: vec!["expr".to_string()],
        body: tokenize("into(nil)"),
      },
    );
    macros.insert(
      "package".to_string(),
      MacroDefinition {
        name: "package".to_string(),
        params: vec!["expr".to_string()],
        body: tokenize("into(nil)"),
      },
    );
    macros.insert(
      "import".to_string(),
      MacroDefinition {
        name: "import".to_string(),
        params: vec!["name".to_string(), "expr".to_string()],
        body: tokenize("local $name = require($expr)"),
      },
    );
    macros.insert(
      "test".to_string(),
      MacroDefinition {
        name: "test".to_string(),
        params: vec!["expr".to_string()],
        body: tokenize("into(nil)"),
      },
    );
    macros.insert(
      "include_bytes".to_string(),
      MacroDefinition {
        name: "include_bytes".to_string(),
        params: vec!["name".to_string(), "expr".to_string()],
        body: tokenize("local $name = bytes_from($expr)"),
      },
    );

    MacroRegistry { macros }
  }

  pub fn define_macro(&mut self, name: String, params: Vec<String>, body: Vec<Token>) {
    self
      .macros
      .insert(name.clone(), MacroDefinition { name, params, body });
  }

  pub fn get_macro(&self, name: &str) -> Option<&MacroDefinition> {
    self.macros.get(name)
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  Number(i64, usize),
  Identifier(String, usize),
  String(String, usize),
  BraceString(String, usize),
  Symbol(String, usize),
  Whitespace(String, usize),
  Macro(usize),
  MacroCall(String, usize),
  MacroParam(String, usize),
  LeftBrace(usize),
  RightBrace(usize),
  LeftParen(usize),
  RightParen(usize),
  Comma(usize),
  EOF(usize),
}

pub struct Lexer {
  pos: usize,
  tokens: usize,
  chars: Vec<char>,
}

impl Lexer {
  pub fn new(input: &str) -> Self {
    Lexer {
      pos: 0,
      tokens: 0,
      chars: input.chars().collect(),
    }
  }

  fn peek_char(&self) -> Option<char> {
    self.chars.get(self.pos).cloned()
  }

  fn next_char(&mut self) -> Option<char> {
    if let Some(ch) = self.peek_char() {
      self.pos += 1;
      Some(ch)
    } else {
      None
    }
  }

  pub fn next_token(&mut self) -> Token {
    if let Some(ch) = self.peek_char() {
      if ch.is_whitespace() {
        return self.read_whitespace();
      } else if ch.is_ascii_digit() {
        return self.read_number();
      } else if ch == '$' {
        return self.read_macro_param();
      } else if ch.is_ascii_alphabetic() || ch == '_' {
        return self.read_identifier_or_macro();
      } else if ch == '"' || ch == '\'' {
        return self.read_string();
      } else if ch == '[' && self.chars.get(self.pos + 1) == Some(&'[') {
        return self.read_string();
      } else if ch == '{' {
        self.next_char();
        return Token::LeftBrace(self.tokens.clone());
      } else if ch == '}' {
        self.next_char();
        return Token::RightBrace(self.tokens.clone());
      } else if ch == '(' {
        self.next_char();
        return Token::LeftParen(self.tokens.clone());
      } else if ch == ')' {
        self.next_char();
        return Token::RightParen(self.tokens.clone());
      } else if ch == ',' {
        self.next_char();
        return Token::Comma(self.tokens.clone());
      } else if ch == '-' && self.chars.get(self.pos + 1) == Some(&'-') {
        return self.read_comment();
      } else {
        return self.read_symbol();
      }
    }
    Token::EOF(self.tokens.clone())
  }

  fn read_whitespace(&mut self) -> Token {
    let mut s = String::new();
    while let Some(ch) = self.peek_char() {
      if ch.is_whitespace() {
        s.push(ch);
        self.next_char();
      } else {
        break;
      }
    }
    Token::Whitespace(s, self.tokens.clone())
  }

  fn read_number(&mut self) -> Token {
    let mut s = String::new();
    while let Some(ch) = self.peek_char() {
      if ch.is_ascii_digit() {
        s.push(ch);
        self.next_char();
      } else {
        break;
      }
    }
    Token::Number(s.parse().unwrap(), self.tokens.clone())
  }

  // fn read_identifier(&mut self) -> Token {
  //   let mut s = String::new();
  //   while let Some(ch) = self.peek_char() {
  //     if ch.is_ascii_alphanumeric() || ch == '_' {
  //       s.push(ch);
  //       self.next_char();
  //     } else {
  //       break;
  //     }
  //   }
  //   Token::Identifier(s, self.tokens.clone())
  // }

  fn read_identifier_or_macro(&mut self) -> Token {
    let mut s = String::new();
    while let Some(ch) = self.peek_char() {
      if ch.is_ascii_alphanumeric() || ch == '_' {
        s.push(ch);
        self.next_char();
      } else {
        break;
      }
    }

    if s == "macro" {
      return Token::Macro(self.tokens.clone());
    }

    if self.peek_char() == Some('!') {
      self.next_char();
      return Token::MacroCall(s, self.tokens.clone());
    }

    Token::Identifier(s, self.tokens.clone())
  }

  fn read_macro_param(&mut self) -> Token {
    self.next_char(); // consume the '$'
    let mut s = String::new();
    while let Some(ch) = self.peek_char() {
      if ch.is_ascii_alphanumeric() || ch == '_' {
        s.push(ch);
        self.next_char();
      } else {
        break;
      }
    }
    Token::MacroParam(s, self.tokens.clone())
  }

  fn read_comment(&mut self) -> Token {
    let mut s = String::new();
    s.push(self.next_char().unwrap()); // first -
    s.push(self.next_char().unwrap()); // second -

    // Read until end of line
    while let Some(ch) = self.peek_char() {
      if ch == '\n' || ch == '\r' {
        break;
      }
      s.push(ch);
      self.next_char();
    }

    Token::Whitespace(s, self.tokens.clone())
  }

  fn read_string(&mut self) -> Token {
    if self.peek_char() == Some('[') && self.chars.get(self.pos + 1) == Some(&'[') {
      self.next_char();
      self.next_char();
      let mut s = String::new();
      while let Some(_) = self.peek_char() {
        if self.peek_char() == Some(']') && self.chars.get(self.pos + 1) == Some(&']') {
          self.next_char();
          self.next_char();
          break;
        } else {
          s.push(self.next_char().unwrap());
        }
      }
      Token::BraceString(s, self.tokens.clone())
    } else {
      let quote = self.next_char().unwrap();
      let mut s = String::new();
      while let Some(ch) = self.next_char() {
        if ch == quote {
          break;
        } else {
          s.push(ch);
        }
      }
      Token::String(s, self.tokens.clone())
    }
  }

  fn read_symbol(&mut self) -> Token {
    let ch = self.next_char().unwrap();
    if ch == '.' && self.peek_char() == Some('.') {
      self.next_char();
      return Token::Symbol("..".into(), self.tokens.clone());
    }
    Token::Symbol(ch.to_string(), self.tokens.clone())
  }
}

pub fn tokenize(input: &str) -> Vec<Token> {
  let mut lexer = Lexer::new(input);
  let mut tokens = Vec::new();

  loop {
    let tok = lexer.next_token();
    if let Token::EOF(_) = tok {
      break;
    }
    tokens.push(tok);
    lexer.tokens += 1;
  }

  tokens
}

fn get_token_string(tok: &Token) -> Option<&String> {
  match tok {
    Token::String(s, _) => Some(s),
    Token::Identifier(s, _) => Some(s),
    _ => None,
  }
}

fn peek_through(
  tokens: &[Token],
  current: usize,
  how_many: isize,
  skip_whitespace: bool,
) -> Option<Token> {
  if tokens.is_empty() {
    return None;
  }

  let mut idx = current as isize;
  let step = if how_many >= 0 { 1 } else { -1 };
  let mut remaining = how_many.abs();

  while remaining > 0 {
    idx += step;

    if idx < 0 || idx >= tokens.len() as isize {
      return None;
    }

    if skip_whitespace && matches!(tokens[idx as usize], Token::Whitespace(_, _)) {
      continue;
    }

    remaining -= 1;
  }

  if idx < 0 || idx >= tokens.len() as isize {
    None
  } else {
    Some(with_idx(&tokens[idx as usize], idx as usize))
  }
}

fn with_idx(tok: &Token, idx: usize) -> Token {
  match tok {
    Token::EOF(_) => Token::EOF(idx),
    Token::String(s, _) => Token::String(s.clone(), idx),
    Token::BraceString(s, _) => Token::BraceString(s.clone(), idx),
    Token::Symbol(s, _) => Token::Symbol(s.clone(), idx),
    Token::Identifier(s, _) => Token::Identifier(s.clone(), idx),
    Token::Number(s, _) => Token::Number(s.clone(), idx),
    Token::Whitespace(s, _) => Token::Whitespace(s.clone(), idx),
    Token::Macro(_) => Token::Macro(idx),
    Token::MacroCall(s, _) => Token::MacroCall(s.clone(), idx),
    Token::MacroParam(s, _) => Token::MacroParam(s.clone(), idx),
    Token::LeftBrace(_) => Token::LeftBrace(idx),
    Token::RightBrace(_) => Token::RightBrace(idx),
    Token::LeftParen(_) => Token::LeftBrace(idx),
    Token::RightParen(_) => Token::RightParen(idx),
    Token::Comma(_) => Token::Comma(idx),
  }
}

// fn get_token_idx(tokens: &[Token], tok: &Token) -> usize {
//   tokens.iter().position(|t| t == tok).unwrap()
// }

fn extract_token_idx(tok: Token) -> usize {
  match tok {
    Token::EOF(i)
    | Token::String(_, i)
    | Token::BraceString(_, i)
    | Token::Symbol(_, i)
    | Token::Identifier(_, i)
    | Token::Number(_, i)
    | Token::Whitespace(_, i)
    | Token::Macro(i)
    | Token::MacroCall(_, i)
    | Token::MacroParam(_, i)
    | Token::LeftBrace(i)
    | Token::RightBrace(i)
    | Token::LeftParen(i)
    | Token::RightParen(i)
    | Token::Comma(i) => i,
  }
}

macro_rules! check_token {
  ($tokens:expr, $i:expr, $how_many:expr, $skip_ws:expr, $pat:pat if $cond:expr => $body:block, $default:expr) => {
    if let Some(next_token) = peek_through($tokens, $i, $how_many, $skip_ws) {
      if let $pat = next_token {
        if $cond { $body } else { $default }
      } else {
        $default
      }
    } else {
      $default
    }
  };

  ($tokens:expr, $i:expr, $how_many:expr, $skip_ws:expr, $pat:pat => $body:block, $default:expr) => {
    if let Some(next_token) = peek_through($tokens, $i, $how_many, $skip_ws) {
      if let $pat = next_token {
        $body
      } else {
        $default
      }
    } else {
      $default
    }
  };
}

fn find_braces(s: &str) -> Vec<(usize, usize)> {
  let mut stack = Vec::new();
  let mut positions = Vec::new();

  for (i, c) in s.char_indices() {
    if c == '{' {
      stack.push(i);
    } else if c == '}' {
      if let Some(start) = stack.pop() {
        positions.push((start, i + 1));
      }
    }
  }

  positions
}

#[derive(Debug, Clone)]
pub struct Compiler {
  macros: MacroRegistry,
  pub defs: HashMap<String, String>,
  pub importmap: HashMap<String, (String, Option<String>, Option<LuluConf>)>,
  pub import: Option<fn(String, String, Option<String>, Option<LuluConf>)>,
  pub last_mod: Option<String>,
  pub env: String,
  pub current_test: Option<String>,
}

impl Compiler {
  pub fn new(import: Option<fn(String, String, Option<String>, Option<LuluConf>)>) -> Self {
    let mut defs = HashMap::new();

    defs.insert("OS".to_string(), std::env::consts::OS.to_lowercase());

    Compiler {
      macros: MacroRegistry::new(),
      defs,
      import,
      importmap: HashMap::new(),
      last_mod: None,
      env: "dev".to_string(),
      current_test: None,
    }
  }

  pub fn compile(&mut self, code: &str, path: Option<String>, conf: Option<LuluConf>) -> String {
    let tokens = tokenize(code);
    let processed_tokens = self.process_macros(tokens, path, conf);
    // println!("{}", self.generate_code(processed_tokens.clone()));
    self.generate_code(processed_tokens)
  }

  fn process_macros(
    &mut self,
    tokens: Vec<Token>,
    path: Option<String>,
    conf: Option<LuluConf>,
  ) -> Vec<Token> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
      match &tokens[i] {
        Token::Macro(_) => {
          i = self.parse_macro_definition(&tokens, i, &mut result);
        }
        Token::MacroCall(name, _) => {
          i = self.expand_macro_call(
            &tokens,
            i,
            &mut result,
            name.clone(),
            path.clone(),
            conf.clone(),
          );
        }
        _ => {
          result.push(tokens[i].clone());
          i += 1;
        }
      }
    }

    result
  }

  fn parse_macro_definition(
    &mut self,
    tokens: &[Token],
    start: usize,
    _result: &mut Vec<Token>,
  ) -> usize {
    let mut i = start + 1;

    while i < tokens.len() && matches!(tokens[i], Token::Whitespace(_, _)) {
      i += 1;
    }

    if i >= tokens.len() || !matches!(tokens[i], Token::LeftBrace(_)) {
      panic!("Expected '{{' after 'macro'");
    }
    i += 1;

    while i < tokens.len() && matches!(tokens[i], Token::Whitespace(_, _)) {
      i += 1;
    }

    let macro_name = match &tokens[i] {
      Token::Identifier(name, _) => name.clone(),
      _ => panic!("Expected macro name after 'macro {{'"),
    };
    i += 1;

    while i < tokens.len() && matches!(tokens[i], Token::Whitespace(_, _)) {
      i += 1;
    }

    if i >= tokens.len() || !matches!(tokens[i], Token::LeftParen(_)) {
      panic!("Expected '(' after macro name");
    }
    i += 1;

    let mut params = Vec::new();
    while i < tokens.len() {
      while i < tokens.len() && matches!(tokens[i], Token::Whitespace(_, _)) {
        i += 1;
      }

      if matches!(tokens[i], Token::RightParen(_)) {
        i += 1;
        break;
      }

      if matches!(tokens[i], Token::MacroParam(_, _)) {
        if let Token::MacroParam(param, _) = &tokens[i] {
          params.push(param.clone());
        }
        i += 1;

        while i < tokens.len() && matches!(tokens[i], Token::Whitespace(_, _)) {
          i += 1;
        }

        if matches!(tokens[i], Token::Comma(_)) {
          i += 1;
        } else if matches!(tokens[i], Token::RightParen(_)) {
          i += 1;
          break;
        }
      } else {
        panic!("Expected macro parameter starting with '$'");
      }
    }

    while i < tokens.len() && matches!(tokens[i], Token::Whitespace(_, _)) {
      i += 1;
    }

    if i >= tokens.len() || !matches!(tokens[i], Token::LeftBrace(_)) {
      panic!("Expected '{{' before macro body");
    }
    i += 1;

    let mut body = Vec::new();
    let mut brace_count = 1;

    while i < tokens.len() && brace_count > 0 {
      match &tokens[i] {
        Token::LeftBrace(_) => {
          brace_count += 1;
          body.push(tokens[i].clone());
        }
        Token::RightBrace(_) => {
          brace_count -= 1;
          if brace_count > 0 {
            body.push(tokens[i].clone());
          }
        }
        _ => {
          body.push(tokens[i].clone());
        }
      }
      i += 1;
    }

    while i < tokens.len() && matches!(tokens[i], Token::Whitespace(_, _)) {
      i += 1;
    }
    if i < tokens.len() && matches!(tokens[i], Token::RightBrace(_)) {
      i += 1;
    }

    self.macros.define_macro(macro_name, params, body);

    i
  }

  fn expand_macro_call(
    &mut self,
    tokens: &[Token],
    start: usize,
    result: &mut Vec<Token>,
    macro_name: String,
    path: Option<String>,
    conf: Option<LuluConf>,
  ) -> usize {
    let s = self.clone();
    let mac = s.macros.get_macro(&macro_name);
    let macro_def = match mac {
      Some(def) => def,
      _ => panic!("Undefined macro: {}", macro_name),
    };

    let mut i = start + 1; // skip macro call token
    let mut args = Vec::new();
    let mut current_arg = Vec::new();
    let mut brace_count = 0;
    let mut paren_count = 0;

    while i < tokens.len() {
      match &tokens[i] {
        Token::LeftParen(_) => {
          paren_count += 1;
          current_arg.push(tokens[i].clone());
          i += 1;
        }
        Token::RightParen(_) => {
          paren_count -= 1;
          current_arg.push(tokens[i].clone());
          i += 1;
        }
        Token::LeftBrace(_) => {
          brace_count += 1;
          if brace_count > 1 {
            current_arg.push(tokens[i].clone());
          }
          i += 1;
        }
        Token::RightBrace(_) => {
          brace_count -= 1;

          if brace_count == 0 && paren_count == 0 {
            i += 1;
            if i >= tokens.len() || !matches!(tokens[i], Token::Comma(_)) {
              break;
            }
          } else {
            current_arg.push(tokens[i].clone());
            i += 1;
          }
        }
        Token::Comma(_) if brace_count == 0 && paren_count == 0 => {
          if !current_arg.is_empty() {
            args.push(current_arg.clone());
            current_arg.clear();
          }
          i += 1;
        }
        Token::Whitespace(_, _) => {
          if !current_arg.is_empty() {
            current_arg.push(tokens[i].clone());
          }
          i += 1;
        }
        Token::Macro(_) => {
          i = self.parse_macro_definition(&tokens, i, &mut current_arg);
        }
        Token::MacroCall(name, _) => {
          i = self.expand_macro_call(
            &tokens,
            i,
            &mut current_arg,
            name.clone(),
            path.clone(),
            conf.clone(),
          );
        }
        _ => {
          current_arg.push(tokens[i].clone());
          i += 1;
        }
      }
    }

    if !current_arg.is_empty() {
      args.push(current_arg);
    }

    let expanded = if macro_name == "lml" {
      tokenize(crate::lml::compile_lml(self.generate_code(args[0].clone()), None).as_str())
    } else if macro_name == "cfg" {
      self.compile_cfg(args, path, conf)
    } else if macro_name == "test" {
      self.compile_tests(args, path, conf)
    } else if macro_name == "match" {
      self.compile_match(args, path, conf)
    } else if macro_name == "import" {
      let mut cargs = args.clone();
      let vname = get_token_string(&args[0][0]).unwrap();
      let cpath = get_token_string(&args[1][0]).unwrap();
      let name = std::path::Path::new(cpath)
        .file_stem()
        .and_then(|s| Some(s.to_string_lossy().to_string()))
        .unwrap_or(vname.clone());

      if let Some(f) = self.import {
        f(name.clone(), cpath.clone(), path.clone(), conf.clone());
      };

      self
        .importmap
        .insert(name.clone(), (cpath.clone(), path.clone(), conf.clone()));
      cargs[1] = vec![Token::String(format!("{}", name), 0)];
      self.substitute_macro_params(&macro_def.body, &macro_def.params, &cargs)
    } else if macro_name == "include_bytes" {
      let mut cargs = args.clone();
      let vname = get_token_string(&args[0][0]).unwrap();
      let name = format!("bytes://{}", vname);
      let cpath = get_token_string(&args[1][0]).unwrap();

      self
        .importmap
        .insert(name.clone(), (cpath.clone(), path.clone(), conf.clone()));
      cargs[1] = vec![Token::String(format!("{}", vname), 0)];
      self.substitute_macro_params(&macro_def.body, &macro_def.params, &cargs)
    } else if macro_name == "package" {
      let name = get_token_string(&args[0][0]).unwrap();
      self.last_mod = Some(name.clone());
      Vec::new()
    } else {
      self.substitute_macro_params(&macro_def.body, &macro_def.params, &args)
    };
    result.extend(expanded);

    i
  }

  fn get_env(&self, name: &String) -> Option<String> {
    if let Some(value) = self.defs.get(name) {
      Some(value.clone())
    } else if let Ok(value) = std::env::var(name) {
      Some(value)
    } else {
      None
    }
  }

  fn compile_cfg(
    &mut self,
    args: Vec<Vec<Token>>,
    path: Option<String>,
    conf: Option<LuluConf>,
  ) -> Vec<Token> {
    let name = self.generate_code(args[0].clone()).trim().to_string();

    let tokens = if name == format!("OS_{}", std::env::consts::OS.to_uppercase()) {
      args[1].clone()
    } else if name == format!("ARCH_{}", std::env::consts::ARCH) {
      args[1].clone()
    } else if name == "set" {
      let c: Vec<String> = self
        .generate_code(args[1].clone())
        .trim()
        .to_string()
        .split('=')
        .collect::<Vec<_>>()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

      self
        .defs
        .insert(c[0].trim().to_string(), c[1].trim().to_string());
      Vec::new()
    } else if let Some(value) = self.get_env(&name) {
      let tokens = &args[1];
      let mut i = 0;

      while i < tokens.len() {
        if matches!(tokens[i], Token::Whitespace(_, _)) {
          i += 1;
        } else {
          break;
        }
      }

      let is_branched = match (tokens.get(i), tokens.get(i + 1)) {
        (Some(Token::Identifier(_, _)), Some(Token::LeftBrace(_))) => true,
        (Some(Token::Identifier(_, _)), Some(Token::Whitespace(_, _))) => {
          matches!(tokens.get(i + 2), Some(Token::LeftBrace(_)))
        }
        (Some(Token::String(_, _)), Some(Token::LeftBrace(_))) => true,
        (Some(Token::String(_, _)), Some(Token::Whitespace(_, _))) => {
          matches!(tokens.get(i + 2), Some(Token::LeftBrace(_)))
        }
        _ => false,
      };

      // println!("{} {}", name, is_branched);

      if is_branched {
        // println!("Exec is branched");
        let mut branches: Vec<(String, Vec<Token>)> = Vec::new();

        while i < tokens.len() {
          let name = match &tokens[i] {
            Token::Identifier(name, _) | Token::String(name, _) => name.clone(),
            Token::Whitespace(_, _) => {
              i += 1;
              continue;
            }
            other => panic!("Expected name identifier, got {:?}", other),
          };
          i += 1;

          if let Token::Whitespace(_, _) = tokens
            .get(i)
            .unwrap_or(&Token::Whitespace("".to_string(), 0))
          {
            i += 1;
          }

          match tokens.get(i) {
            Some(Token::LeftBrace(_)) => i += 1,
            other => panic!("Expected '{{' after name, got {:?}", other),
          }

          let start = i;
          let mut brace_count = 1;
          while brace_count > 0 {
            match &tokens[i] {
              Token::LeftBrace(_) => brace_count += 1,
              Token::RightBrace(_) => brace_count -= 1,
              _ => {}
            }
            i += 1;
          }
          let end = i - 1;
          let branch_tokens = tokens[start..end].to_vec();
          branches.push((name, branch_tokens));
        }

        let current = value.clone().to_lowercase();
        if let Some((_, tok)) = branches.iter().find(|(os, _)| os.to_lowercase() == current) {
          tok.clone()
        } else {
          match args.get(2) {
            Some(arg2) => arg2.clone(),
            _ => Vec::new(),
          }
        }
      } else {
        // println!("Exec non branched");
        tokens.clone()
      }
    } else {
      match args.get(2) {
        Some(arg2) => arg2.clone(),
        _ => Vec::new(),
      }
    };

    self.process_macros(tokens, path, conf)
  }

  fn compile_tests(
    &mut self,
    args: Vec<Vec<Token>>,
    path: Option<String>,
    conf: Option<LuluConf>,
  ) -> Vec<Token> {
    if self.env == "test" {
      let mut branches: Vec<(String, Vec<Token>)> = Vec::new();
      let tokens = &args[0];
      let mut i = 0;

      while i < tokens.len() {
        let name = match &tokens[i] {
          Token::Identifier(name, _) | Token::String(name, _) => name.clone(),
          Token::Whitespace(_, _) => {
            i += 1;
            continue;
          }
          other => panic!("Expected name identifier, got {:?}", other),
        };
        i += 1;

        if let Token::Whitespace(_, _) = tokens
          .get(i)
          .unwrap_or(&Token::Whitespace("".to_string(), 0))
        {
          i += 1;
        }

        match tokens.get(i) {
          Some(Token::LeftBrace(_)) => i += 1,
          other => panic!("Expected '{{' after name, got {:?}", other),
        }

        let start = i;
        let mut brace_count = 1;
        while brace_count > 0 {
          match &tokens[i] {
            Token::LeftBrace(_) => brace_count += 1,
            Token::RightBrace(_) => brace_count -= 1,
            _ => {}
          }
          i += 1;
        }
        let end = i - 1;
        let mut branch_tokens = Vec::new();
        branch_tokens.extend(tokenize(format!("local {} = function()", name).as_str()));
        branch_tokens.extend(tokens[start..end].to_vec());
        branch_tokens.extend(tokenize(format!("end\nlocal ok_{name}, err_{name} = pcall({name})\nif ok_{name} then\n  print(\"Finished test: {name}\")\nelse\n  print(\"Test {name} failed due to:\", err_{name})\nend\n\n", name = name).as_str()));
        branches.push((name, branch_tokens));
      }

      self.process_macros(
        if let Some(current) = self.current_test.clone() {
          if let Some((_, tok)) = branches.iter().find(|(os, _)| os.to_lowercase() == current) {
            tok.clone()
          } else {
            match args.get(2) {
              Some(arg2) => arg2.clone(),
              _ => Vec::new(),
            }
          }
        } else {
          let mut v: Vec<Token> = Vec::new();
          for (_, value) in branches.iter() {
            v.extend(value.clone());
          }
          v
        },
        path,
        conf,
      )
    } else {
      Vec::new()
    }
  }

  fn capture_expression(&mut self, tokens: &[Token], start: usize) -> (Vec<Token>, usize) {
    let mut out = Vec::new();
    let mut i = start;
    let mut paren = 0;

    while i < tokens.len() {
      match &tokens[i] {
        Token::LeftBrace(_) if paren == 0 => break,

        Token::LeftParen(_) => {
          paren += 1;
          out.push(tokens[i].clone());
        }
        Token::RightParen(_) => {
          if paren == 0 {
            break;
          }
          paren -= 1;
          out.push(tokens[i].clone());
        }

        _ => out.push(tokens[i].clone()),
      }
      i += 1;
    }

    (out, i)
  }

  fn compile_match(
    &mut self,
    args: Vec<Vec<Token>>,
    path: Option<String>,
    conf: Option<LuluConf>,
  ) -> Vec<Token> {
    let mut branches: Vec<(Vec<Token>, Vec<Token>)> = Vec::new();
    let value = &args[0];
    let tokens = &args[1];
    let mut i = 0;

    while i < tokens.len() {
      let (expr_tokens, next_i) = self.capture_expression(tokens, i);
      if expr_tokens.is_empty() {
        panic!(
          "Expected match pattern (identifier, string, number, call, or table) at {:?}",
          i
        );
      }
      i = next_i;

      // println!("{:?}", expr_tokens);

      if let Some(Token::Whitespace(_, _)) = tokens.get(i) {
        i += 1;
      }

      match tokens.get(i) {
        Some(Token::LeftBrace(_)) => i += 1,
        other => panic!("Expected '{{' after match pattern, got {:?}", other),
      }

      let start = i;
      let mut brace_count = 1;
      while brace_count > 0 {
        match &tokens[i] {
          Token::LeftBrace(_) => brace_count += 1,
          Token::RightBrace(_) => brace_count -= 1,
          _ => {}
        }
        i += 1;
      }
      let end = i - 1;

      if let Some(Token::Whitespace(_, _)) = tokens.get(i) {
        i += 1;
      }

      let mut branch_tokens = Vec::new();

      if expr_tokens.iter().any(|t| matches!(t, Token::Identifier(v, _) if *v == "_".to_string())) {
        branch_tokens.extend(tokenize("else"));
        branch_tokens.extend(tokens[start..end].to_vec());
        branches.push((expr_tokens.clone(), branch_tokens));
        continue;
      }
      
      if branches.len() > 0 {
        branch_tokens.extend(tokenize("elseif "));
      } else {
        branch_tokens.extend(tokenize("if "));
      }

      if !expr_tokens.iter().any(|t| matches!(t, Token::Identifier(v, _) if *v == "val".to_string())) {
        branch_tokens.extend(tokenize("val == "));
      }
      branch_tokens.extend(expr_tokens.clone());
      branch_tokens.extend(tokenize(" then "));
      branch_tokens.extend(tokens[start..end].to_vec());

      branches.push((expr_tokens.clone(), branch_tokens));

    }

    self.process_macros(
      {
        let mut v: Vec<Token> = Vec::new();
        v.extend(tokenize("(function(val)\n"));
        for (_, value) in branches.iter() {
          v.extend(value.clone());
        }
        v.extend(tokenize("end\n"));
        v.extend(tokenize("end)("));
        v.extend(value.clone());
        v.extend(tokenize(")"));
        if !v.iter().any(|t| matches!(t, Token::Identifier(s, _) if s == "return")) {
          v.insert(0, Token::Symbol(";".into(), 0));
        }
        v
      },
      path,
      conf,
    )
  }

  fn substitute_macro_params(
    &self,
    body: &[Token],
    param_names: &[String],
    args: &[Vec<Token>],
  ) -> Vec<Token> {
    let mut result = Vec::new();

    for token in body {
      match token {
        Token::MacroParam(param, _) => {
          if let Some(index) = param_names.iter().position(|p| p == param) {
            if index < args.len() {
              result.extend(args[index].clone());
            } else {
              panic!("Not enough arguments for macro parameter: ${}", param);
            }
          } else {
            panic!("Unknown macro parameter: ${}", param);
          }
        }
        _ => {
          result.push(token.clone());
        }
      }
    }

    result
  }

  fn generate_code(&self, tokens: Vec<Token>) -> String {
    let mut result = String::new();
    let mut i = 0;

    while i < tokens.len() {
      let token = &tokens[i];

      match token {
        Token::Identifier(name, _) if name == "f" => {
          check_token!(&tokens, i, 1, true, Token::String(s, _) => {
            let string_token = s.replace("{{", "\0LEFT_BRACE\0")
                        .replace("}}", "\0RIGHT_BRACE\0");

            let mut formatted = String::new();
            let mut last = 0;

            for (start, end) in find_braces(&string_token) {
              if start > last {
                let literal = &string_token[last..start];
                formatted.push_str(&format!("\"{}\" .. ", literal));
              }

              let expr = &string_token[start + 1..end - 1]; // skip {}
              formatted.push_str(&format!("({}) .. ", expr));

              last = end;
            }

            if last < string_token.len() {
                formatted.push_str(&format!("\"{}\"", &string_token[last..]));
            } else {
              if formatted.ends_with(" .. ") {
                formatted.truncate(formatted.len() - 4);
              }
            }

            result.push_str(&formatted.replace("\0LEFT_BRACE\0", "{")
                     .replace("\0RIGHT_BRACE\0", "}"));
            i += 1;
          }, result.push_str(name));
        }
        Token::Identifier(name, _) => {
          result.push_str(name);
        }
        Token::Number(n, _) => {
          result.push_str(&n.to_string());
        }
        Token::String(s, _) => {
          result.push_str(&format!("\"{}\"", s));
        }
        Token::BraceString(s, _) => {
          result.push_str(&format!("[[{}]]", s));
        }
        Token::Symbol(sym, _) if sym == "&" => {
          check_token!(&tokens, i, 1, true, Token::String(current_token, _)=> {
            result.push_str(format!("ptr_of({:?})", current_token).as_str());
            i += 1;
          }, check_token!(&tokens, i, 1, true, Token::Number(current_token, _) => {
            result.push_str(format!("ptr_of({:?})", current_token).as_str());
            i += 1;
          }, check_token!(&tokens, i, 1, true, Token::Identifier(current_token, _) => {
            result.push_str(format!("ptr_of({})", current_token).as_str());
            i += 1;
          }, result.push_str(sym))));
        }
        Token::Symbol(sym, _) if sym == "*" => {
          check_token!(&tokens, i, 1, false, Token::Identifier(current_token, ind) => {
            check_token!(&tokens, ind, 1, true, Token::Symbol(char, ind) if char == '='.to_string() => {
              let next_token = &peek_through(&tokens, ind, 1, true).unwrap_or(Token::EOF(0));
              result.push_str(format!("ptr_set({}, {})", current_token, match next_token {
                 Token::String(s, _) => format!("{:?}", s),
                 Token::Identifier(s, _) => format!("{}", s),
                 Token::Number(s, _) => format!("{}", s),
                 _ => panic!("You can only set a pointer to a preset value")
              }).as_str());
              i = extract_token_idx(next_token.clone());
            }, {
              result.push_str(format!("ptr_deref({})", current_token).as_str());
              i += 1;
            });
          }, result.push_str(sym));
        }
        Token::Symbol(sym, _) => {
          result.push_str(sym);
        }
        Token::LeftBrace(_) => {
          result.push_str("{");
        }
        Token::RightBrace(_) => {
          result.push_str("}");
        }
        Token::LeftParen(_) => {
          result.push_str("(");
        }
        Token::RightParen(_) => {
          result.push_str(")");
        }
        Token::Comma(_) => {
          result.push_str(",");
        }
        Token::Whitespace(ws, _) => {
          result.push_str(ws);
        }
        Token::Macro(_) | Token::MacroCall(_, _) | Token::MacroParam(_, _) => {
          // already parsed
        }
        Token::EOF(_) => {}
      }

      i += 1;
    }

    result
  }
}

pub fn compile(code: &str) -> String {
  let mut compiler = Compiler::new(None);
  compiler.compile(code, None, None)
}

pub fn wrap_macros(input: &str) -> String {
  if let Some(start) = input.find("macros = {") {
    let mut brace_count = 0;
    let mut end = start;
    for (i, ch) in input[start..].char_indices() {
      match ch {
        '{' => brace_count += 1,
        '}' => {
          brace_count -= 1;
          if brace_count == 0 {
            end = start + i;
            break;
          }
        }
        _ => {}
      }
    }

    let before = &input[..start];
    let body = &input[start + "macros = {".len()..end].trim();
    let after = &input[end + 1..]; // skip final `}`

    let mut wrapped_macros = Vec::new();
    let mut current = String::new();
    let mut inner_brace_count = 0;
    let mut paren_count = 0;

    for c in body.chars() {
      match c {
        '{' => {
          inner_brace_count += 1;
          current.push(c);
        }
        '}' => {
          inner_brace_count -= 1;
          current.push(c);
        }
        '(' => {
          paren_count += 1;
          current.push(c);
        }
        ')' => {
          paren_count -= 1;
          current.push(c);
        }
        ',' if inner_brace_count == 0 && paren_count == 0 => {
          let m = current.trim();
          if !m.is_empty() {
            wrapped_macros.push(format!("macro {{\n{}\n}}", m));
          }
          current.clear();
        }
        _ => current.push(c),
      }
    }

    let m = current.trim();
    if !m.is_empty() {
      wrapped_macros.push(format!("macro {{\n{}\n}}", m));
    }

    let wrapped_body = wrapped_macros.join("\n");
    let macros_block = format!("macros = [[\n{}\n]]", wrapped_body);

    format!("{}{}\n{}", before.trim_end(), macros_block, after)
  } else {
    input.to_string()
  }
}
