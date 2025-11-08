use std::collections::HashMap;

use crate::{conf::LuluConf, ops::std::STD_MODULES};

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

// const KEYWORDS: [&str;3] = ["class", "enum", "match"];

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
        body: tokenize("\nfor $item in ipairs($iterator) do\n$block\nend\n"),
      },
    );
    macros.insert(
      "for_pairs".to_string(),
      MacroDefinition {
        name: "for_pairs".to_string(),
        params: vec![
          "key".to_string(),
          "value".to_string(),
          "iterator".to_string(),
          "block".to_string(),
        ],
        body: tokenize("\nfor $key, $value in pairs($iterator) do\n$block\nend\n"),
      },
    );
    macros.insert(
      "when".to_string(),
      MacroDefinition {
        name: "when".to_string(),
        params: vec![
          "condition".to_string(),
          "then_block".to_string(),
          "_otherwise".to_string(),
        ],
        body: tokenize("\nif $condition then\n$then_block\nelse\n$_otherwise\nend\n"),
      },
    );
    macros.insert(
      "repeat_n".to_string(),
      MacroDefinition {
        name: "repeat_n".to_string(),
        params: vec!["start".to_string(), "times".to_string(), "body".to_string()],
        body: tokenize("\nfor i = $start, $times do\n$body\nend\n"),
      },
    );
    macros.insert(
      "try_catch".to_string(),
      MacroDefinition {
        name: "try_catch".to_string(),
        params: vec![
          "try_block".to_string(),
          "_catch_block".to_string()
        ],
        body: tokenize("\nlocal ok, err = pcall(function()\n$try_block\nend)\nif not ok then\n$_catch_block\nend\n"),
      },
    );
    macros.insert(
      "lazy".to_string(),
      MacroDefinition {
        name: "lazy".to_string(),
        params: vec!["name".to_string(), "expr".to_string()],
        body: tokenize("\nlocal __lazy_$name\nfunction get_$name()\nif not __lazy_$name then __lazy_$name = $expr end\nreturn __lazy_$name\nend\n"),
      },
    );
    macros.insert(
      "guard".to_string(),
      MacroDefinition {
        name: "guard".to_string(),
        params: vec!["condition".to_string(), "error".to_string()],
        body: tokenize("\nif not ($condition) then $error end\n"),
      },
    );
    macros.insert(
      "class".to_string(),
      MacroDefinition {
        name: "class".to_string(),
        params: vec![
          "name".to_string(),
          "methods".to_string(),
          "_constructor".to_string(),
        ],
        body: tokenize("into(nil)"),
      },
    );
    macros.insert(
      "spread".to_string(),
      MacroDefinition {
        name: "spread".to_string(),
        params: vec!["name".to_string(), "methods".to_string()],
        body: tokenize("into(nil)"),
      },
    );
    macros.insert(
      "collect".to_string(),
      MacroDefinition {
        name: "collect".to_string(),
        params: vec!["methods".to_string()],
        body: tokenize("into(nil)"),
      },
    );
    macros.insert(
      "enum".to_string(),
      MacroDefinition {
        name: "enum".to_string(),
        params: vec!["name".to_string(), "methods".to_string()],
        body: tokenize("into(nil)"),
      },
    );
    macros.insert(
      "decorator".to_string(),
      MacroDefinition {
        name: "decorator".to_string(),
        params: vec!["methods".to_string()],
        body: tokenize("into(nil)"),
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
        params: vec!["expr".to_string()],
        body: tokenize("bytes_from($expr)"),
      },
    );
    macros.insert(
      "include_string".to_string(),
      MacroDefinition {
        name: "include_string".to_string(),
        params: vec!["expr".to_string()],
        body: tokenize("include_bytes! { $expr }:to_string()"),
      },
    );

    if let Ok(modules) = STD_MODULES.read() {
      for (_, module) in modules.iter() {
        for (name, params, body) in module.macros.clone() {
          macros.insert(
            name.clone(),
            MacroDefinition {
              name: name,
              params: params,
              body: tokenize(&body),
            },
          );
        }
      }
    }

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    // if KEYWORDS.contains(&s.as_str()) {
    //   return Token::MacroCall(s, self.tokens.clone());
    // }

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

    if (ch == '=' || ch == '-') && self.peek_char() == Some('>') {
      return Token::Symbol(
        format!("{}{}", ch, self.next_char().unwrap()),
        self.tokens.clone(),
      );
    }

    if ch == '-' && self.peek_char() == Some('<') {
      return Token::Symbol(
        format!("{}{}", ch, self.next_char().unwrap()),
        self.tokens.clone(),
      );
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

macro_rules! pass_until_block_end {
  ($tokens:expr, $i:expr, $depth:expr) => {
    let mut block_depth = 0;
    while $i < $tokens.len() {
      match $tokens[$i] {
        Token::Identifier(ref id, _)
          if id == "function" || id == "while" || id == "for" || id == "until" || id == "repeat" || id == "do" || id == "if" =>
        {
          block_depth += 1
        }
        Token::Symbol(ref sym, _) if sym == "=>" =>
        {
          block_depth += 1
        }
        Token::Identifier(ref id, _) if id == "end" => {
          if block_depth == $depth {
            break;
          } else {
            block_depth -= 1;
          }
        }
        _ => {}
      }
      $i += 1;
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
    defs.insert("ARCH".to_string(), std::env::consts::ARCH.to_lowercase());
    defs.insert(
      "FAMILY".to_string(),
      std::env::consts::FAMILY.to_lowercase(),
    );

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

  fn process_leftbrace(&mut self, i: usize, tokens: Vec<Token>) -> (usize, String, Vec<Token>) {
    let mut j = i;
    let mut name_decorators: Vec<Token> = Vec::new();
    let mut name = String::new();

    while j < tokens.len() && matches!(tokens[j], Token::Whitespace(_, _)) {
      j += 1;
    }

    while j < tokens.len() && matches!(&tokens[j], Token::Symbol(s, _) if s == "@") {
      let start = j;
      j += 1;

      // decorator name + possible arguments
      while j < tokens.len() && matches!(&tokens[j], Token::Identifier(_, _) | Token::LeftParen(_))
      {
        if matches!(&tokens[j], Token::LeftParen(_)) {
          let mut paren_depth = 1;
          j += 1;
          while j < tokens.len() && paren_depth > 0 {
            match &tokens[j] {
              Token::LeftParen(_) => paren_depth += 1,
              Token::RightParen(_) => paren_depth -= 1,
              _ => {}
            }
            j += 1;
          }
          break;
        } else {
          j += 1;
        }
      }

      let end = j;
      name_decorators.extend_from_slice(&tokens[start..end]);

      while j < tokens.len() && matches!(tokens[j], Token::Whitespace(_, _)) {
        j += 1;
      }
    }

    if j < tokens.len() && matches!(&tokens[j], Token::Identifier(i, _) if i != "end") {
      if let Some(n) = get_token_string(&tokens[j]) {
        name = n.clone();
      }
      j += 1;
    }

    if j < tokens.len() && matches!(&tokens[j], Token::Symbol(s, _) if s == ":") {
      if j < tokens.len() && matches!(&tokens[j + 1], Token::Identifier(i, _) if i != "end") {
        name = format!("{}:{}", name, get_token_string(&tokens[j + 1]).unwrap());
        j += 2;
      }
    }

    return (j, name, name_decorators);
  }

  fn process_lulib_import(
    &mut self,
    i: usize,
    tokens: &Vec<Token>,
    path: Option<String>,
    conf: Option<LuluConf>,
  ) -> Option<(usize, String, String)> {
    let mut j = i;

    while j < tokens.len() && matches!(tokens[j], Token::Whitespace(_, _)) {
      j += 1;
    }

    if j < tokens.len() && matches!(&tokens[j], Token::LeftBrace(_)) {
      j += 1;
      while j < tokens.len() && matches!(tokens[j], Token::Whitespace(_, _)) {
        j += 1;
      }

      let mut name = String::new();
      let mut cpath = String::new();

      if j < tokens.len() && matches!(&tokens[j], Token::Identifier(i, _) if i != "end") {
        if let Some(n) = get_token_string(&tokens[j]) {
          name = n.clone();
        }
        j += 1;
      }

      while j < tokens.len() && matches!(tokens[j], Token::Whitespace(_, _)) {
        j += 1;
      }

      if j < tokens.len() && matches!(&tokens[j], Token::Comma(_)) {
        j += 1;
      }

      while j < tokens.len() && matches!(tokens[j], Token::Whitespace(_, _)) {
        j += 1;
      }

      if j < tokens.len() && matches!(&tokens[j], Token::String(_, _)) {
        if let Some(n) = get_token_string(&tokens[j]) {
          cpath = n.clone();
        }
        j += 1;
      }

      let mut modn = String::new();
      if !cpath.is_empty() {
        let modname = crate::util::normalize_name(&cpath);
        modn = modname.clone();

        self
          .importmap
          .insert(modname.clone(), (cpath.clone(), path.clone(), conf.clone()));
      }

      while j < tokens.len() && matches!(tokens[j], Token::Whitespace(_, _)) {
        j += 1;
      }

      j += 1;

      return Some((j, name, modn));
    }

    None
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
        Token::LeftBrace(_) => {
          // find the matching right brace
          let mut j = i + 1;
          let mut brace_depth = 1;
          while j < tokens.len() && brace_depth > 0 {
            match &tokens[j] {
              Token::LeftBrace(_) => brace_depth += 1,
              Token::RightBrace(_) => brace_depth -= 1,
              _ => {}
            }
            j += 1;
          }
          let brace_end = j;

          while j < tokens.len() && matches!(tokens[j], Token::Whitespace(_, _)) {
            j += 1;
          }

          if j < tokens.len() && matches!(&tokens[j], Token::Symbol(s, _) if s == "->") {
            j += 1;

            let (j, name, name_decorators) = self.process_leftbrace(j, tokens.clone());

            let inner_tokens = &tokens[i + 1..brace_end - 1];

            let mut tokens_to_pass = Vec::new();
            tokens_to_pass.extend(name_decorators);
            tokens_to_pass.extend(tokenize(&name));
            if inner_tokens.len() > 0 {
              tokens_to_pass.push(Token::LeftParen(0));
              tokens_to_pass.extend_from_slice(inner_tokens);
              tokens_to_pass.push(Token::RightParen(0));
            }
            let new_tokens = self.compile_class(vec![tokens_to_pass], path.clone(), conf.clone());
            result.extend(new_tokens);
            i = j + 1;
            continue;
          }

          if j < tokens.len() && matches!(&tokens[j], Token::Symbol(s, _) if s == "-<") {
            j += 1;

            let (j, name, name_decorators) = self.process_leftbrace(j, tokens.clone());

            let inner_tokens = &tokens[i + 1..brace_end - 1];

            let mut tokens_to_pass = Vec::new();
            tokens_to_pass.extend(name_decorators);
            tokens_to_pass.extend(tokenize(&name));
            let new_tokens = self.compile_enum(
              vec![tokens_to_pass, inner_tokens.to_vec()],
              path.clone(),
              conf.clone(),
            );
            result.extend(new_tokens);
            i = j + 1;
            continue;
          }

          // normal push
          result.push(tokens[i].clone());
          i += 1;
        }
        Token::Identifier(ident, _) if ident == "using" => {
          let mut j = i + 2;
          if j < tokens.len()
            && matches!(&tokens[j], Token::Identifier(ident, _) if ident == "lulib")
          {
            j += 1;

            if let Some((j, name, modn)) =
              self.process_lulib_import(j, &tokens, path.clone(), conf.clone())
            {
              result.extend(vec![
                Token::Identifier("using ".to_string(), 1),
                Token::Symbol("{ ".to_string(), 1),
                Token::Identifier("lulib".to_string(), 1),
                Token::Symbol("(".to_string(), 1),
                Token::String(name, 1),
                Token::Comma(1),
                Token::String(modn, 1),
                Token::Symbol(")".to_string(), 1),
                Token::Symbol(" }".to_string(), 1),
              ]);

              i = j;
              continue;
            }
          }

          result.push(tokens[i].clone());
          i += 1;
        }
        Token::Identifier(ident, _) if ident == "lulib" => {
          if let Some((j, name, modn)) =
            self.process_lulib_import(i + 1, &tokens, path.clone(), conf.clone())
          {
            result.extend(vec![
              Token::Identifier("lulib".to_string(), 1),
              Token::Symbol("(".to_string(), 1),
              Token::String(name, 1),
              Token::Comma(1),
              Token::String(modn, 1),
              Token::Symbol(")".to_string(), 1),
            ]);

            i = j;
            continue;
          }

          result.push(tokens[i].clone());
          i += 1;
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
          if brace_count > 1 || paren_count > 0 {
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
        Token::Symbol(sym, _) if sym == ";" => {
          if brace_count == 0 && paren_count == 0 {
            i += 1;
            break;
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
    } else if macro_name == "class" {
      self.compile_class(args, path, conf)
    } else if macro_name == "spread" {
      self.compile_spread(args, path, conf)
    } else if macro_name == "collect" {
      self.compile_collect(args, path, conf)
    } else if macro_name == "enum" {
      self.compile_enum(args, path, conf)
    } else if macro_name == "decorator" {
      self.compile_decorator(args, path, conf)
    } else if macro_name == "import" {
      let mut cargs = args.clone();
      let cpath = get_token_string(&args[1][0]).unwrap();
      let name = crate::util::normalize_name(cpath);

      // f = function
      // idk why i name things weird
      if let Some(f) = self.import {
        f(name.clone(), cpath.clone(), path.clone(), conf.clone());
      };

      self
        .importmap
        .insert(name.clone(), (cpath.clone(), path.clone(), conf.clone()));
      cargs[1] = vec![Token::String(format!("{}", name), 0)];
      self.substitute_macro_params(
        &macro_def.body,
        &macro_def.params,
        &cargs,
        path.clone(),
        conf.clone(),
      )
    } else if macro_name == "include_bytes" {
      let cpath = get_token_string(&args[0][0]).unwrap();
      let name = format!("bytes://{}", crate::util::normalize_name(cpath));

      self
        .importmap
        .insert(name.clone(), (cpath.clone(), path.clone(), conf.clone()));
      self.substitute_macro_params(
        &macro_def.body,
        &macro_def.params,
        &[vec![Token::String(format!("{}", name), 0)]],
        path.clone(),
        conf.clone(),
      )
    } else if macro_name == "package" {
      let name = get_token_string(&args[0][0]).unwrap();
      self.last_mod = Some(name.clone());
      Vec::new()
    } else {
      self.substitute_macro_params(
        &macro_def.body,
        &macro_def.params,
        &args,
        path.clone(),
        conf.clone(),
      )
    };
    result.extend(expanded);

    i
  }

  fn compile_spread(
    &mut self,
    args: Vec<Vec<Token>>,
    path: Option<String>,
    conf: Option<LuluConf>,
  ) -> Vec<Token> {
    if args.len() < 2 {
      panic!("spread! expects two arguments: variable and pattern");
    }

    let source_tokens = &args[0];
    let pattern_tokens = &args[1];
    let source_name = self.generate_code(source_tokens.clone());
    let items = self.extract_pattern_items(pattern_tokens);

    let mut lua = String::new();
    let mut index = 1;

    for (i, item) in items.iter().enumerate() {
      let trimmed = item.trim();

      // Spread (...something)
      if trimmed.starts_with("...") {
        let name = trimmed.trim_start_matches("...");
        let (start, end) = {
          let start = index;
          let end = if i == items.len() - 1 {
            format!("#{}", source_name)
          } else {
            let remaining = items.len() - (i + 1);
            format!("#{} - {}", source_name, remaining)
          };
          (start, end)
        };

        if name.contains('.') {
          lua.push_str(&format!(
            "{} = {{ unpack({}, {}, {}) }}\n",
            name, source_name, start, end
          ));
        } else {
          lua.push_str(&format!(
            "local {} = {{ unpack({}, {}, {}) }}\n",
            name, source_name, start, end
          ));
        }
        index += 1;
        continue;
      }

      if trimmed == "_" {
        index += 1;
        continue;
      }

      if trimmed.contains('.') && !trimmed.contains(':') {
        lua.push_str(&format!("{} = {}[{}]\n", trimmed, source_name, index));
        index += 1;
        continue;
      }

      if let Some(colon_index) = trimmed.find(':') {
        let (var, prop) = trimmed.split_at(colon_index);
        let var = var.trim();
        let prop = prop.trim_start_matches(':').trim();
        lua.push_str(&format!("local {} = {}.{}\n", var, source_name, prop));
        continue;
      }

      if trimmed.starts_with('&') {
        let var = trimmed.trim_start_matches('&');
        lua.push_str(&format!("local {} = {}.{}\n", var, source_name, var));
        continue;
      }

      lua.push_str(&format!("local {} = {}[{}]\n", trimmed, source_name, index));
      index += 1;
    }

    self.process_macros(tokenize(lua.as_str()), path, conf)
  }

  fn compile_collect(
    &mut self,
    args: Vec<Vec<Token>>,
    path: Option<String>,
    conf: Option<LuluConf>,
  ) -> Vec<Token> {
    if args.is_empty() {
      panic!("collect! expects at least one argument block");
    }

    let pattern_tokens = &args[0];
    let items = self.extract_pattern_items(pattern_tokens);

    let mut has_spreads = false;
    let mut parts = Vec::new();

    for item in &items {
      let trimmed = item.trim();
      if trimmed.is_empty() {
        continue;
      }

      if trimmed.starts_with("...") || trimmed.starts_with("..") {
        has_spreads = true;
        continue;
      }

      if let Some(eq_index) = trimmed.find('=') {
        let (key, val) = trimmed.split_at(eq_index);
        parts.push(format!(
          "{} = {}",
          key.trim(),
          val.trim_start_matches('=').trim()
        ));
      } else {
        parts.push(format!("{} = {}", trimmed, trimmed));
      }
    }

    if !has_spreads {
      let table = format!("{{ {} }}", parts.join(", "));
      return self.process_macros(tokenize(&table), path, conf);
    }

    let mut lua = String::new();
    lua.push_str("(function()\n  local _tbl = {");
    if !parts.is_empty() {
      lua.push_str(&parts.join(", "));
    }
    lua.push_str("}\n");

    for item in &items {
      let trimmed = item.trim();
      if trimmed.starts_with("...") {
        let name = trimmed.trim_start_matches("...");
        lua.push_str(&format!(
          "  for _,v in ipairs({}) do table.insert(_tbl, v) end\n",
          name
        ));
      } else if trimmed.starts_with("..") {
        let name = trimmed.trim_start_matches("..");
        lua.push_str(&format!(
          "  for k,v in pairs({}) do _tbl[k] = v end\n",
          name
        ));
      }
    }

    lua.push_str("  return _tbl\nend)()");

    self.process_macros(tokenize(&lua), path, conf)
  }

  fn extract_pattern_items(&self, tokens: &[Token]) -> Vec<String> {
    use Token::*;
    let mut result = Vec::new();
    let mut current = std::string::String::new();

    for token in tokens {
      match token {
        Comma(_) => {
          if !current.trim().is_empty() {
            result.push(current.trim().to_string());
          }
          current.clear();
        }
        _ => {
          current.push_str(self.generate_code(vec![token.clone()]).as_str());
        }
      }
    }

    if !current.trim().is_empty() {
      result.push(current.trim().to_string());
    }

    result
  }

  fn compile_enum(
    &mut self,
    args: Vec<Vec<Token>>,
    path: Option<String>,
    conf: Option<LuluConf>,
  ) -> Vec<Token> {
    if args.len() < 2 {
      panic!("enum! expects two arguments: name and variants block");
    }

    let decl_tokens: &_ = &args[0];
    let variants_tokens = &args[1];

    // Parse enum decorators
    let mut i = 0;
    let mut enum_decorators: Vec<Vec<Token>> = Vec::new();
    while i < decl_tokens.len() {
      if let Some(Token::Symbol(s, _)) = decl_tokens.get(i) {
        if s == "@" {
          i += 1; // Skip '@'
          let mut decorator_tokens = Vec::new();
          if let Some(Token::Identifier(_, _)) = decl_tokens.get(i) {
            decorator_tokens.push(decl_tokens[i].clone());
            i += 1;
            if let Some(Token::LeftParen(_)) = decl_tokens.get(i) {
              let start_paren = i;
              let mut paren_count = 1;
              i += 1;
              while i < decl_tokens.len() && paren_count > 0 {
                if let Token::LeftParen(_) = &decl_tokens[i] {
                  paren_count += 1;
                } else if let Token::RightParen(_) = &decl_tokens[i] {
                  paren_count -= 1;
                }
                i += 1;
              }
              decorator_tokens.extend_from_slice(&decl_tokens[start_paren..i]);
            }
            enum_decorators.push(decorator_tokens);
          }
          while i < decl_tokens.len() && matches!(decl_tokens.get(i), Some(Token::Whitespace(_, _)))
          {
            i += 1;
          }
          continue;
        }
      }
      break;
    }

    let remaining_decl_tokens = &decl_tokens[i..];
    let enum_name = self
      .generate_code(remaining_decl_tokens.to_vec())
      .trim()
      .to_string();

    // Variant parsing
    let mut variants: Vec<(String, Option<Vec<String>>, Vec<Vec<Token>>)> = Vec::new();
    let mut variant_groups: Vec<Vec<Token>> = Vec::new();
    let mut current_variant_tokens: Vec<Token> = Vec::new();

    let mut paren_depth = 0;
    for token in variants_tokens {
      match token {
        Token::LeftParen(_) => {
          paren_depth += 1;
          current_variant_tokens.push(token.clone());
        }
        Token::RightParen(_) => {
          if paren_depth > 0 {
            paren_depth -= 1;
          }
          current_variant_tokens.push(token.clone());
        }
        Token::Comma(_) if paren_depth == 0 => {
          // split only if not inside parentheses
          if !current_variant_tokens.is_empty() {
            variant_groups.push(current_variant_tokens);
            current_variant_tokens = Vec::new();
          }
        }
        _ => {
          current_variant_tokens.push(token.clone());
        }
      }
    }

    if !current_variant_tokens.is_empty() {
      variant_groups.push(current_variant_tokens);
    }

    for tokens in variant_groups {
      let mut i = 0;
      // skip leading whitespace
      while i < tokens.len() && matches!(tokens[i], Token::Whitespace(_, _)) {
        i += 1;
      }
      if i >= tokens.len() {
        continue;
      }

      let mut variant_decorators: Vec<Vec<Token>> = Vec::new();
      loop {
        while i < tokens.len() && matches!(tokens[i], Token::Whitespace(_, _)) {
          i += 1;
        }
        if i >= tokens.len() {
          break;
        }

        if let Some(Token::Symbol(s, _)) = tokens.get(i) {
          if s == "@" {
            i += 1; // Skip '@'
            let mut decorator_tokens = Vec::new();
            if let Some(Token::Identifier(_, _)) = tokens.get(i) {
              decorator_tokens.push(tokens[i].clone());
              i += 1;
              if let Some(Token::LeftParen(_)) = tokens.get(i) {
                let start_paren = i;
                let mut paren_count = 1;
                i += 1;
                while i < tokens.len() && paren_count > 0 {
                  if let Token::LeftParen(_) = &tokens[i] {
                    paren_count += 1;
                  } else if let Token::RightParen(_) = &tokens[i] {
                    paren_count -= 1;
                  }
                  i += 1;
                }
                decorator_tokens.extend_from_slice(&tokens[start_paren..i]);
              }
              variant_decorators.push(decorator_tokens);
              continue;
            }
          }
        }
        break;
      }

      while i < tokens.len() && matches!(tokens[i], Token::Whitespace(_, _)) {
        i += 1;
      }

      let variant_name = if let Some(Token::Identifier(name, _)) = tokens.get(i) {
        i += 1;
        name.clone()
      } else {
        if i >= tokens.len() {
          continue;
        }
        panic!("Expected variant name, found {:?}", tokens.get(i));
      };

      while i < tokens.len() && matches!(tokens[i], Token::Whitespace(_, _)) {
        i += 1;
      }

      let mut variant_args: Option<Vec<String>> = None;
      if let Some(Token::LeftParen(_)) = tokens.get(i) {
        let start_paren = i;
        let mut paren_count = 1;
        i += 1;
        while i < tokens.len() && paren_count > 0 {
          if let Token::LeftParen(_) = &tokens[i] {
            paren_count += 1;
          } else if let Token::RightParen(_) = &tokens[i] {
            paren_count -= 1;
          }
          i += 1;
        }
        let args_tokens = &tokens[start_paren + 1..i - 1];
        let args_str = self.generate_code(args_tokens.to_vec());
        let args: Vec<String> = if args_str.is_empty() {
          Vec::new()
        } else {
          args_str.split(',').map(|s| s.trim().to_string()).collect()
        };
        variant_args = Some(args);
      }

      variants.push((variant_name, variant_args, variant_decorators));
    }

    let mut lua = String::new();
    let mut variant_decorator_lua = String::new();

    lua.push_str(&format!(
      "{name} = make_enum(\"{name}\")\n",
      name = enum_name
    ));

    for (vname, args, decorators) in &variants {
      if let Some(args) = args {
        let args_list = args
          .into_iter()
          .map(|x| format!("\"{}\"", x))
          .collect::<Vec<String>>()
          .join(", ");
        lua.push_str(&format!(
            "{enum}.{vname} = make_enum_var_dyn({enum}, '{vname}', {{ {args_list} }})\n",
            enum = enum_name,
            vname = vname,
            args_list = args_list
        ));
      } else {
        lua.push_str(&format!(
            "{enum}.{vname} = make_enum_var({enum}, '{vname}')\n",
            enum = enum_name,
            vname = vname
        ));
      }

      for decorator in decorators.iter().rev() {
        let decorator_str = self.generate_code(decorator.clone());
        variant_decorator_lua.push_str(&format!(
          "{0}.{1} = {2}({0}, {0}.{1}, \"{1}\")\n",
          enum_name, vname, decorator_str
        ));
      }
    }

    let mut tokenized = tokenize(&lua);
    tokenized.extend(tokenize(&variant_decorator_lua));

    if args.len() > 2 {
      let mut branches: Vec<(Vec<Token>, Vec<Token>)> = Vec::new();
      let tokens = &args[2];
      let mut i = 0;

      while i < tokens.len() {
        let (expr_tokens, next_i) = self.capture_extra_expression(tokens, i);
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

        branches.push((expr_tokens.clone(), tokens[start..end].to_vec()));
      }

      for (func, block) in branches.iter() {
        let name = func[0].clone();
        let args = func[1..].to_vec();

        let processed_block = self.process_macros(block.clone(), path.clone(), conf.clone());

        tokenized.extend(vec![
          Token::Whitespace("\n".to_string(), 0),
          Token::Identifier(enum_name.clone(), 0),
          Token::Symbol(".".to_string(), 0),
          Token::Identifier("func".to_string(), 0),
          Token::Symbol(".".to_string(), 0),
          name.clone(),
          Token::Whitespace(" ".to_string(), 0),
          Token::Symbol("=".to_string(), 0),
          Token::Whitespace(" ".to_string(), 0),
          Token::Identifier("function".to_string(), 0),
        ]);

        tokenized.extend(args);
        tokenized.extend(vec![Token::Whitespace("\n".to_string(), 0)]);
        tokenized.extend(processed_block);
        tokenized.extend(vec![Token::Whitespace("\n".to_string(), 0)]);
        tokenized.extend(vec![
          Token::Identifier("end".to_string(), 0),
          Token::Whitespace("\n".to_string(), 0),
        ]);
      }
    }

    // println!("{}", self.generate_code(tokenized.clone()));

    let mut decorator_code = String::new();
    for decorator in enum_decorators.iter().rev() {
      let decorator_str = self.generate_code(decorator.clone());
      decorator_code.push_str(&format!(
        "{0} = {1}({0}, \"{0}\")\n",
        enum_name, decorator_str
      ));
    }
    tokenized.extend(tokenize(&decorator_code));

    self.process_macros(tokenized, path, conf)
  }

  fn capture_until_comma(&mut self, tokens: &[Token], start: usize) -> (Vec<Token>, usize) {
    let mut out = Vec::new();
    let mut i = start;
    let mut paren = 0;
    let mut brace = 0;

    while i < tokens.len() {
      match &tokens[i] {
        Token::LeftParen(_) => {
          paren += 1;
          out.push(tokens[i].clone());
        }
        Token::RightParen(_) => {
          if paren > 0 {
            paren -= 1;
            out.push(tokens[i].clone());
          } else {
            break;
          }
        }
        Token::LeftBrace(_) => {
          brace += 1;
          out.push(tokens[i].clone());
        }
        Token::RightBrace(_) => {
          if brace > 0 {
            brace -= 1;
            out.push(tokens[i].clone());
          } else {
            break;
          }
        }
        Token::Comma(_) if paren == 0 && brace == 0 => break,
        _ => out.push(tokens[i].clone()),
      }
      i += 1;
    }

    (out, i)
  }

  fn compile_class(
    &mut self,
    args: Vec<Vec<Token>>,
    path: Option<String>,
    conf: Option<LuluConf>,
  ) -> Vec<Token> {
    if args.len() < 1 {
      panic!("Class expects atleast a name");
    }

    let decl_tokens = &args[0];
    let block_tokens = if args.len() < 2 {
      &Vec::new()
    } else if args.len() > 2 {
      &args[2]
    } else {
      &args[1]
    };
    let constructor_block = if args.len() > 2 {
      args[1].clone()
    } else {
      Vec::new()
    };

    // Parse class decorators
    let mut i = 0;
    let mut class_decorators: Vec<Vec<Token>> = Vec::new();
    while i < decl_tokens.len() {
      if let Some(Token::Symbol(s, _)) = decl_tokens.get(i) {
        if s == "@" {
          i += 1; // Skip '@'
          let mut decorator_tokens = Vec::new();
          if let Some(Token::Identifier(_, _)) = decl_tokens.get(i) {
            decorator_tokens.push(decl_tokens[i].clone());
            i += 1;
            if let Some(Token::LeftParen(_)) = decl_tokens.get(i) {
              let start_paren = i;
              let mut paren_count = 1;
              i += 1;
              while i < decl_tokens.len() && paren_count > 0 {
                if let Token::LeftParen(_) = &decl_tokens[i] {
                  paren_count += 1;
                } else if let Token::RightParen(_) = &decl_tokens[i] {
                  paren_count -= 1;
                }
                i += 1;
              }
              decorator_tokens.extend_from_slice(&decl_tokens[start_paren..i]);
            }
            class_decorators.insert(0, decorator_tokens);
          }
          while i < decl_tokens.len() && matches!(decl_tokens.get(i), Some(Token::Whitespace(_, _)))
          {
            i += 1;
          }
          continue;
        }
      }
      break;
    }

    let remaining_decl_tokens = &decl_tokens[i..];
    let decl_str = self
      .generate_code(remaining_decl_tokens.to_vec())
      .trim()
      .to_string();

    #[allow(unused_assignments)]
    let mut class_name = String::new();
    let mut parent_name = None;
    let mut constructor_args: Vec<String> = Vec::new();

    let buffer = decl_str.clone();

    if let Some(paren_idx) = buffer.find('(') {
      class_name = buffer[..paren_idx].trim().to_string();
      let args_str = buffer[paren_idx + 1..buffer.len() - 1].trim();
      if !args_str.is_empty() {
        constructor_args = args_str.split(',').map(|s| s.trim().to_string()).collect();
      }
    } else {
      class_name = buffer;
    }

    if let Some(idx) = class_name.find(':') {
      parent_name = Some(class_name[idx + 1..].trim().to_string());
      class_name = class_name[..idx].trim().to_string();
    }

    let mut self_assignments = String::new();

    // Re-tokenize constructor argument list directly
    let mut constructor_arg_tokens = Vec::new();
    if !constructor_args.is_empty() {
      let joined = constructor_args.join(", ");
      constructor_arg_tokens = tokenize(&joined);
    }

    let mut i = 0;
    let mut arg_index = 1;

    while i < constructor_arg_tokens.len() {
      // Skip whitespace and commas
      while i < constructor_arg_tokens.len()
        && matches!(
          &constructor_arg_tokens[i],
          Token::Whitespace(_, _) | Token::Comma(_)
        )
      {
        i += 1;
      }
      if i >= constructor_arg_tokens.len() {
        break;
      }

      // Collect decorators
      let mut decorators = Vec::new();
      while i < constructor_arg_tokens.len() {
        match &constructor_arg_tokens[i] {
          Token::Symbol(s, _) if s == "@" => {
            i += 1;
            let mut decorator_tokens = Vec::new();
            if let Some(Token::Identifier(_, _)) = constructor_arg_tokens.get(i) {
              decorator_tokens.push(constructor_arg_tokens[i].clone());
              i += 1;

              // Handle @decorator(...)
              if let Some(Token::LeftParen(_)) = constructor_arg_tokens.get(i) {
                let start = i;
                let mut depth = 1;
                i += 1;
                while i < constructor_arg_tokens.len() && depth > 0 {
                  match &constructor_arg_tokens[i] {
                    Token::LeftParen(_) => depth += 1,
                    Token::RightParen(_) => depth -= 1,
                    _ => {}
                  }
                  i += 1;
                }
                decorator_tokens.extend_from_slice(&constructor_arg_tokens[start..i]);
              }
              decorators.push(decorator_tokens);
            }
          }
          Token::Whitespace(_, _) => i += 1,
          _ => break,
        }
      }

      // Now read the actual argument identifier (could be `self.x`, `&y`, or just `z`)
      let mut name_tokens = Vec::new();
      while i < constructor_arg_tokens.len() {
        match &constructor_arg_tokens[i] {
          Token::Comma(_) | Token::Whitespace(_, _) => break,
          Token::RightParen(_) => break,
          _ => {
            name_tokens.push(constructor_arg_tokens[i].clone());
            i += 1;
          }
        }
      }

      if name_tokens.is_empty() {
        continue;
      }

      let mut name_str = self.generate_code(name_tokens.clone()).trim().to_string();

      if name_str == "_" {
        arg_index += 1;
        continue;
      }

      let res_str = if name_str.starts_with("#") {
        name_str = name_str[1..].to_string();
        format!("type(args[{arg_index}]) == \"table\" and args[{arg_index}].{name_str} or nil")
      } else {
        let ind = arg_index;
        arg_index += 1;
        format!("args[{ind}]")
      };

      let (assign_target, assign_expr) = if name_str.contains('.') {
        (name_str.clone(), res_str)
      } else if name_str.starts_with('&') {
        (name_str.trim_start_matches('&').to_string(), res_str)
      } else {
        (format!("self.{name_str}"), res_str)
      };

      if !decorators.is_empty() {
        let mut expr = assign_expr.clone();
        for deco in decorators {
          let deco_str = self.generate_code(deco);
          expr = format!("{deco_str}(self, {expr}, \"{name_str}\")");
        }
        self_assignments.push_str(&format!("{assign_target} = {expr}\n"));
      } else {
        self_assignments.push_str(&format!("{assign_target} = {assign_expr}\n"));
      }
    }

    let init_line = if let Some(parent) = parent_name.clone() {
      format!("setmetatable({{}}, {{ __index = {} }})", parent)
    } else {
      format!("{{}}")
    };

    let index_parent = if let Some(parent) = parent_name.clone() {
      format!(", {}", parent)
    } else {
      "".to_string()
    };

    let call_parent = if let Some(parent) = parent_name.clone() {
      if constructor_block
        .iter()
        .any(|x| matches!(x, Token::Identifier(x, _) if x == "super"))
      {
        format!(
          "local super = function(...) {}.__construct(self, false, ...) end",
          parent
        )
      } else {
        format!("{}.__construct(self, false, ...)", parent)
      }
    } else {
      "".to_string()
    };
    let mut constructor_block_str = constructor_block.clone();

    if !constructor_block.is_empty() && matches!(constructor_block[0], Token::LeftParen(_)) {
      // 1. Extract everything inside ( ... )
      let mut inner = Vec::new();
      let mut depth = 0;
      let mut i = 0;

      while i < constructor_block.len() {
        match &constructor_block[i] {
          Token::LeftParen(_) => {
            depth += 1;
            if depth > 1 {
              inner.push(constructor_block[i].clone());
            }
          }
          Token::RightParen(_) => {
            depth -= 1;
            if depth == 0 {
              i += 1;
              break;
            } else {
              inner.push(constructor_block[i].clone());
            }
          }
          _ => {
            if depth > 0 {
              inner.push(constructor_block[i].clone());
            }
          }
        }
        i += 1;
      }

      let inner_str = self.generate_code(inner);

      let spread_code = format!("spread! args, {{ {} }}", inner_str);
      let mut spread_tokens = tokenize(&spread_code);

      spread_tokens.extend_from_slice(&constructor_block[i..]);

      constructor_block_str = self.process_macros(spread_tokens, path.clone(), conf.clone());
    }

    let constructor_code = format!(
      r#"
function {name}:__construct(is_first, ...)
  local args = {{...}}
  {call_parent}
  {assignments}
  {constructor_block}
  if self.__call_init and is_first then self:__call_init(...) end
end
"#,
      name = class_name,
      call_parent = call_parent,
      constructor_block = self.generate_code(constructor_block_str),
      assignments = self_assignments,
    );

    let lua_code = format!(
      r#"{name} = make_class({init_line}{index_parent})

{constructor_code}
"#,
      name = class_name,
      init_line = init_line,
      index_parent = index_parent,
      constructor_code = constructor_code
    );

    let mut tokens = tokenize(lua_code.as_str());

    let mut branches: Vec<(Vec<Token>, Vec<Token>, Vec<Vec<Token>>)> = Vec::new();
    let mut i = 0;

    while i < block_tokens.len() {
      while i < block_tokens.len() && matches!(&block_tokens[i], Token::Whitespace(_, _)) {
        i += 1;
      }
      if i >= block_tokens.len() {
        break;
      }

      let mut method_decorators: Vec<Vec<Token>> = Vec::new();
      while i < block_tokens.len() {
        if let Some(Token::Symbol(s, _)) = block_tokens.get(i) {
          if s == "@" {
            i += 1;
            let mut decorator_tokens = Vec::new();
            if let Some(Token::Identifier(_, _)) = block_tokens.get(i) {
              decorator_tokens.push(block_tokens[i].clone());
              i += 1;
              if let Some(Token::LeftParen(_)) = block_tokens.get(i) {
                let start_paren = i;
                let mut paren_count = 1;
                i += 1;
                while i < block_tokens.len() && paren_count > 0 {
                  if let Token::LeftParen(_) = &block_tokens[i] {
                    paren_count += 1;
                  } else if let Token::RightParen(_) = &block_tokens[i] {
                    paren_count -= 1;
                  }
                  i += 1;
                }
                decorator_tokens.extend_from_slice(&block_tokens[start_paren..i]);
              }
              method_decorators.push(decorator_tokens);
            }
            while i < block_tokens.len()
              && matches!(block_tokens.get(i), Some(Token::Whitespace(_, _)))
            {
              i += 1;
            }
            continue;
          }
        }
        break;
      }

      let (expr_tokens, is_decl, next_i) = self.capture_expression_or_declaration(block_tokens, i);
      if expr_tokens.is_empty() {
        if i < block_tokens.len() {
          panic!(
            "Expected match pattern (identifier, string, number, call, or table) at {:?}",
            i
          );
        } else {
          break;
        }
      }
      i = next_i;

      if is_decl {
        let field_name = if let Token::Identifier(name, _) = &expr_tokens[0] {
          name
        } else {
          panic!("Expected identifier for field declaration");
        };

        let mut eq_index = None;
        for (idx, tok) in expr_tokens.iter().enumerate() {
          if let Token::Symbol(s, _) = tok {
            if s == "=" {
              eq_index = Some(idx);
              break;
            }
          }
        }

        if let Some(eq_idx) = eq_index {
          let (value_tokens, idx) = self.capture_until_comma(&block_tokens, i + eq_idx);
          let value_str = self.generate_code(value_tokens);
          i = idx + 1;
          let field_code = format!("{}.{field_name} = {}\n", class_name, value_str);
          tokens.extend(tokenize(&field_code));
        } else {
          panic!("Malformed field declaration, missing '='");
        }

        continue;
      }

      if let Some(Token::Whitespace(_, _)) = block_tokens.get(i) {
        i += 1;
      }

      match block_tokens.get(i) {
        Some(Token::LeftBrace(_)) => i += 1,
        other => panic!("Expected '{{' after match pattern, got {:?}", other),
      }

      let start = i;
      let mut brace_count = 1;
      while brace_count > 0 {
        match &block_tokens[i] {
          Token::LeftBrace(_) => brace_count += 1,
          Token::RightBrace(_) => brace_count -= 1,
          _ => {}
        }
        i += 1;
      }
      let end = i - 1;

      if let Some(Token::Whitespace(_, _)) = block_tokens.get(i) {
        i += 1;
      }

      branches.push((
        expr_tokens.clone(),
        block_tokens[start..end].to_vec(),
        method_decorators,
      ));
    }

    for (func, block, decorators) in branches.iter() {
      let name = func[0].clone();
      let mut args_tokens = if func.len() > 1 {
        func[1..].to_vec()
      } else {
        Vec::new()
      };

      let mut method_args: Vec<Token> = Vec::new();
      let mut param_decorators: Vec<(String, Vec<Token>)> = Vec::new();

      if !args_tokens.is_empty() {
        args_tokens = args_tokens[1..args_tokens.len() - 1].to_vec();
      }

      let mut i = 0;
      while i < args_tokens.len() {
        while i < args_tokens.len()
          && (matches!(&args_tokens[i], Token::Whitespace(_, _))
            || matches!(&args_tokens[i], Token::Comma(_)))
        {
          i += 1;
        }
        if i >= args_tokens.len() {
          break;
        }

        let mut current_arg_decorators = Vec::new();
        while i < args_tokens.len() {
          if let Some(Token::Symbol(s, _)) = args_tokens.get(i) {
            if s == "@" {
              i += 1;
              let mut decorator_tokens = Vec::new();
              if let Some(Token::Identifier(_, _)) = args_tokens.get(i) {
                decorator_tokens.push(args_tokens[i].clone());
                i += 1;
                if let Some(Token::LeftParen(_)) = args_tokens.get(i) {
                  let start_paren = i;
                  let mut paren_count = 1;
                  i += 1;
                  while i < args_tokens.len() && paren_count > 0 {
                    if let Token::LeftParen(_) = &args_tokens[i] {
                      paren_count += 1;
                    } else if let Token::RightParen(_) = &args_tokens[i] {
                      paren_count -= 1;
                    }
                    i += 1;
                  }
                  decorator_tokens.extend_from_slice(&args_tokens[start_paren..i]);
                }
                current_arg_decorators.push(decorator_tokens);
              }
              while i < args_tokens.len()
                && matches!(args_tokens.get(i), Some(Token::Whitespace(_, _)))
              {
                i += 1;
              }
              continue;
            }
          }
          break;
        }

        if i < args_tokens.len() {
          if let Token::Identifier(arg_name, _) = &args_tokens[i] {
            method_args.push(args_tokens[i].clone());
            method_args.push(Token::Comma(0));

            for decorator in current_arg_decorators {
              param_decorators.push((arg_name.clone(), decorator));
            }
          } else {
            method_args.push(args_tokens[i].clone());
          }
          i += 1;
        }
      }

      if !method_args.is_empty() {
        if matches!(method_args[method_args.len() - 1], Token::Comma(_)) {
          method_args.pop();
        } else if matches!(method_args[method_args.len() - 1], Token::RightParen(_)) {
          method_args.pop();
          if !method_args.is_empty() {
            if matches!(method_args[method_args.len() - 1], Token::Comma(_)) {
              method_args.pop();
            }
          }
        }
      }

      let mut args_with_parens = vec![Token::LeftParen(0)];
      args_with_parens.extend(method_args);
      args_with_parens.push(Token::RightParen(0));
      let args = args_with_parens;

      let mut param_decorator_code = String::new();
      for (arg_name, decorator) in param_decorators {
        let decorator_str = self.generate_code(decorator);
        param_decorator_code.push_str(&format!(
          "{0} = {1}(self, {0}, \"{0}\")\n",
          arg_name, decorator_str
        ));
      }
      let param_decorator_tokens = tokenize(&param_decorator_code);

      let processed_block = self.process_macros(block.clone(), path.clone(), conf.clone());

      tokens.extend(vec![
        Token::Whitespace("\n".to_string(), 0),
        Token::Identifier("function".to_string(), 0),
        Token::Whitespace(" ".to_string(), 0),
        Token::Identifier(class_name.clone(), 0),
        Token::Symbol(":".to_string(), 0),
        name.clone(),
      ]);

      tokens.extend(args);
      tokens.extend(vec![Token::Whitespace("\n".to_string(), 0)]);
      tokens.extend(param_decorator_tokens);
      tokens.extend(processed_block);
      tokens.extend(vec![Token::Whitespace("\n".to_string(), 0)]);
      tokens.extend(vec![
        Token::Identifier("end".to_string(), 0),
        Token::Whitespace("\n".to_string(), 0),
      ]);

      for decorator in decorators.iter().rev() {
        let decorator_str = self.generate_code(decorator.clone());
        let method_name_str = self.generate_code(vec![name.clone()]);
        let decorated_method = format!(
          "{0}.{1} = {2}({0}, {0}.{1}, \"{1}\")\n",
          class_name, method_name_str, decorator_str
        );
        tokens.extend(tokenize(&decorated_method));
      }
    }

    let mut decorator_code = String::new();
    for decorator in class_decorators.iter().rev() {
      let decorator_str = self.generate_code(decorator.clone());
      decorator_code.push_str(&format!(
        "{0} = {1}({0}, \"{0}\")\n",
        class_name, decorator_str
      ));
    }
    tokens.extend(tokenize(&decorator_code));

    self.process_macros(tokens, path, conf)
  }

  fn capture_extra_expression(&mut self, tokens: &[Token], start: usize) -> (Vec<Token>, usize) {
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

  fn capture_expression(&self, tokens: &[Token], start: usize) -> (Vec<Token>, usize) {
    let mut i = start;
    while i < tokens.len() && matches!(tokens[i], Token::Whitespace(_, _)) {
      i += 1;
    }

    if i >= tokens.len() {
      return (vec![], i);
    }

    let start_expr = i;

    if let Token::LeftParen(_) = &tokens[i] {
      let mut paren_count = 1;
      i += 1;
      while i < tokens.len() && paren_count > 0 {
        match &tokens[i] {
          Token::LeftParen(_) => paren_count += 1,
          Token::RightParen(_) => paren_count -= 1,
          _ => {}
        }
        i += 1;
      }
      return (tokens[start_expr..i].to_vec(), i);
    }

    if let Token::Identifier(_, _) = &tokens[i] {
      i += 1;
      return (tokens[start_expr..i].to_vec(), i);
    }

    (vec![], start)
  }

  fn compile_decorator(
    &mut self,
    args: Vec<Vec<Token>>,
    path: Option<String>,
    conf: Option<LuluConf>,
  ) -> Vec<Token> {
    if args.is_empty() {
      panic!("decorator! expects a body");
    }

    let body_tokens = &args[0];

    let mut common_body = vec![];
    let mut class_method_sig = vec![];
    let mut class_method_body = vec![];
    let mut class_sig = vec![];
    let mut class_body = vec![];
    let mut enum_variant_sig = vec![];
    let mut enum_variant_body = vec![];
    let mut enum_sig = vec![];
    let mut enum_body = vec![];
    let mut param_sig = vec![];
    let mut param_body = vec![];
    let mut function_sig = vec![];
    let mut function_body = vec![];

    let mut i = 0;
    while i < body_tokens.len() {
      while i < body_tokens.len() && matches!(&body_tokens[i], Token::Whitespace(_, _)) {
        i += 1;
      }
      if i >= body_tokens.len() {
        break;
      }

      let (signature_tokens, next_i) = self.capture_expression(body_tokens, i);
      i = next_i;

      while i < body_tokens.len() && matches!(&body_tokens[i], Token::Whitespace(_, _)) {
        i += 1;
      }

      if i >= body_tokens.len() {
        break;
      }

      match body_tokens.get(i) {
        Some(Token::LeftBrace(_)) => i += 1,
        other => panic!(
          "Expected '{{' after decorator branch signature, got {:?}",
          other
        ),
      }

      let start = i;
      let mut brace_count = 1;
      while i < body_tokens.len() && brace_count > 0 {
        match &body_tokens[i] {
          Token::LeftBrace(_) => brace_count += 1,
          Token::RightBrace(_) => brace_count -= 1,
          _ => {}
        }
        i += 1;
      }
      let end = i - 1;
      let current_body = body_tokens[start..end].to_vec();

      let sig_str = self
        .generate_code(signature_tokens.clone())
        .trim()
        .to_string();

      if sig_str == "_" {
        common_body = current_body;
      } else if sig_str.starts_with('(') && sig_str.ends_with(')') {
        let inner_sig_str = &sig_str[1..sig_str.len() - 1];
        let params: Vec<&str> = inner_sig_str.split(',').map(|s| s.trim()).collect();

        if params.len() == 1 {
          if params[0] == "_class" {
            class_sig = signature_tokens;
            class_body = current_body;
          } else if params[0] == "_enum" {
            enum_sig = signature_tokens;
            enum_body = current_body;
          } else if params[0] == "_function" {
            function_sig = signature_tokens;
            function_body = current_body;
          }
        } else if params.len() == 2 {
          if params[0] == "_class" && params[1] == "method" {
            class_method_sig = signature_tokens;
            class_method_body = current_body;
          } else if params[0] == "_enum" && params[1] == "variant" {
            enum_variant_sig = signature_tokens;
            enum_variant_body = current_body;
          } else if params[0] == "_self" && params[1] == "value" {
            param_sig = signature_tokens;
            param_body = current_body;
          }
        }
      }
    }

    let mut lua_code = String::from(
      "function(...)\n    local arg1, arg2, arg3 = select(1, ...)\n local name = arg2\n if arg3 then name = arg3 end\n",
    );

    if !common_body.is_empty() {
      lua_code.push_str(self.generate_code(common_body).as_str());
      lua_code.push('\n');
    }

    let mut first_if = true;
    let mut if_or_elseif = || {
      if first_if {
        first_if = false;
        "if"
      } else {
        "elseif"
      }
    };

    if !param_body.is_empty() {
      let sig_str = self.generate_code(param_sig.clone());
      let inner_sig_str = &sig_str[1..sig_str.len() - 1];
      let body = self.generate_code(param_body);
      lua_code.push_str(&format!(
        "    {} type(arg1) == \"table\" and arg1.__class and arg3 then\n",
        if_or_elseif()
      ));
      lua_code.push_str(&format!("      local {} = arg1, arg2\n", inner_sig_str));
      lua_code.push_str(&body);
    }

    if !function_body.is_empty() {
      let sig_str = self.generate_code(function_sig.clone());
      let inner_sig_str = &sig_str[1..sig_str.len() - 1];
      let body = self.generate_code(function_body);
      lua_code.push_str(&format!(
        "    {} type(arg1) == \"function\" and not arg3 then\n",
        if_or_elseif()
      ));
      lua_code.push_str(&format!("      local {} = arg1\n", inner_sig_str));
      lua_code.push_str(&body);
    }

    if !class_method_body.is_empty() {
      let sig_str = self.generate_code(class_method_sig.clone());
      let inner_sig_str = &sig_str[1..sig_str.len() - 1];
      let body = self.generate_code(class_method_body);
      lua_code.push_str(&format!(
        "    {} type(arg1) == \"table\" and arg1.__call_init and arg3 then\n",
        if_or_elseif()
      ));
      lua_code.push_str(&format!("      local {} = arg1, arg2\n", inner_sig_str));
      lua_code.push_str(&body);
    }

    if !class_body.is_empty() {
      let sig_str = self.generate_code(class_sig.clone());
      let inner_sig_str = &sig_str[1..sig_str.len() - 1];
      let body = self.generate_code(class_body);
      lua_code.push_str(&format!(
        "    {} type(arg1) == \"table\" and arg1.__call_init and not arg3 then\n",
        if_or_elseif()
      ));
      lua_code.push_str(&format!("      local {} = arg1\n", inner_sig_str));
      lua_code.push_str(&body);
    }

    if !enum_variant_body.is_empty() {
      let sig_str = self.generate_code(enum_variant_sig.clone());
      let inner_sig_str = &sig_str[1..sig_str.len() - 1];

      let mut variant_common_body = vec![];
      let mut variant_static_body = vec![];
      let mut variant_dynamic_body = vec![];
      let mut i = 0;
      while i < enum_variant_body.len() {
        while i < enum_variant_body.len()
          && matches!(&enum_variant_body[i], Token::Whitespace(_, _))
        {
          i += 1;
        }
        if i >= enum_variant_body.len() {
          break;
        }

        let (sub_sig_tokens, next_i) = self.capture_expression(&enum_variant_body, i);
        i = next_i;

        while i < enum_variant_body.len()
          && matches!(&enum_variant_body[i], Token::Whitespace(_, _))
        {
          i += 1;
        }

        if i >= enum_variant_body.len() {
          break;
        }

        match enum_variant_body.get(i) {
          Some(Token::LeftBrace(_)) => i += 1,
          other => panic!(
            "Expected '{{' after enum variant decorator branch signature, got {:?}",
            other
          ),
        }

        let start = i;
        let mut brace_count = 1;
        while i < enum_variant_body.len() && brace_count > 0 {
          match &enum_variant_body[i] {
            Token::LeftBrace(_) => brace_count += 1,
            Token::RightBrace(_) => brace_count -= 1,
            _ => {}
          }
          i += 1;
        }
        let end = i - 1;
        let sub_body = enum_variant_body[start..end].to_vec();
        let sub_sig_str = self.generate_code(sub_sig_tokens).trim().to_string();

        if sub_sig_str == "_" {
          variant_common_body = sub_body;
        } else if sub_sig_str == "static" {
          variant_static_body = sub_body;
        } else if sub_sig_str == "dynamic" {
          variant_dynamic_body = sub_body;
        }
      }

      lua_code.push_str(&format!(
        "    {} type(arg1) == \"table\" and arg1.__is_enum and arg3 then\n",
        if_or_elseif()
      ));
      lua_code.push_str(&format!("      local {} = arg1, arg2\n", inner_sig_str));

      if !variant_common_body.is_empty() {
        lua_code.push_str(&self.generate_code(variant_common_body));
      }
      lua_code.push_str("\n      if type(arg2) == \"function\" then\n");
      if !variant_dynamic_body.is_empty() {
        lua_code.push_str(&self.generate_code(variant_dynamic_body));
      }
      lua_code.push_str("\n      else\n");
      if !variant_static_body.is_empty() {
        lua_code.push_str(&self.generate_code(variant_static_body));
      }
      lua_code.push_str("\n      end\n");
    }

    if !enum_body.is_empty() {
      let sig_str = self.generate_code(enum_sig.clone());
      let inner_sig_str = &sig_str[1..sig_str.len() - 1];
      let body = self.generate_code(enum_body);
      lua_code.push_str(&format!(
        "    {} type(arg1) == \"table\" and arg1.__is_enum and not arg3 then\n",
        if_or_elseif()
      ));
      lua_code.push_str(&format!("      local {} = arg1\n", inner_sig_str));
      lua_code.push_str(&body);
    }

    if !first_if {
      lua_code.push_str("    end\n");
    }

    lua_code.push_str("  end");

    self.process_macros(tokenize(&lua_code), path, conf)
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

  fn capture_expression_or_declaration(
    &mut self,
    tokens: &[Token],
    start: usize,
  ) -> (Vec<Token>, bool, usize) {
    let mut out = Vec::new();
    let mut i = start;
    let mut paren = 0;
    let mut is_declaration = false;

    while i < tokens.len() {
      match &tokens[i] {
        // stop at block start
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

        Token::Symbol(s, _) if s == "=" && paren == 0 => {
          is_declaration = true;
          out.push(tokens[i].clone());
          break;
        }

        _ => out.push(tokens[i].clone()),
      }
      i += 1;
    }

    (out, is_declaration, i)
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
      let (expr_tokens, next_i) = self.capture_extra_expression(tokens, i);
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

      if expr_tokens
        .iter()
        .any(|t| matches!(t, Token::Identifier(v, _) if *v == "_".to_string()))
      {
        branch_tokens.extend(tokenize("else "));
        branch_tokens.extend(tokens[start..end].to_vec());
        branches.push((expr_tokens.clone(), branch_tokens));
        continue;
      }

      if branches.len() > 0 {
        branch_tokens.extend(tokenize("elseif "));
      } else {
        branch_tokens.extend(tokenize("if "));
      }

      let or_parts: Vec<Vec<Token>> = expr_tokens
        .as_slice()
        .split(|tok| matches!(tok, Token::Identifier(s, _) if s == "or"))
        .map(|slice| {
          if slice.is_empty() {
            return Vec::new();
          }
          let first = slice
            .iter()
            .position(|t| !matches!(t, Token::Whitespace(_, _)))
            .unwrap_or(slice.len());
          let last = slice
            .iter()
            .rposition(|t| !matches!(t, Token::Whitespace(_, _)))
            .unwrap_or(0);

          if first > last {
            return Vec::new();
          }
          slice[first..=last].to_vec()
        })
        .filter(|v| !v.is_empty())
        .collect();

      for (idx, part) in or_parts.iter().enumerate() {
        let mut current_part = part.clone();
        let has_not = if let Some(Token::Identifier(s, _)) = current_part.get(0) {
          s == "not"
        } else {
          false
        };

        if has_not {
          current_part.remove(0);
          if !current_part.is_empty() {
            if let Token::Whitespace(_, _) = &current_part[0] {
              current_part.remove(0);
            }
          }
        }

        let iscustom = !current_part
          .iter()
          .any(|t| matches!(t, Token::Identifier(v, _) if *v == "val".to_string()));

        if has_not {
          branch_tokens.extend(tokenize("not "));
        }

        if iscustom {
          branch_tokens.extend(tokenize("iseq(val, "));
        }
        branch_tokens.extend(current_part);
        if iscustom {
          branch_tokens.extend(tokenize(")"));
        }
        if idx < or_parts.len() - 1 {
          branch_tokens.extend(tokenize(" or "));
        }
      }
      branch_tokens.extend(tokenize(" then "));
      branch_tokens.extend(tokens[start..end].to_vec());

      branches.push((expr_tokens.clone(), branch_tokens));
    }

    self.process_macros(
      {
        let mut v: Vec<Token> = Vec::new();
        let mut is_returned = false;
        v.extend(tokenize("(function(val)\n"));
        for (_, value) in branches.iter() {
          if !is_returned {
            is_returned = value
              .iter()
              .any(|t| matches!(t, Token::Identifier(s, _) if s == "return"));
          }
          v.extend(value.clone());
        }
        v.extend(tokenize("end\n"));
        v.extend(tokenize("end)("));
        v.extend(value.clone());
        v.extend(tokenize(")"));
        if !is_returned {
          v.insert(0, Token::Whitespace("\n".into(), 0));
          v.insert(0, Token::Symbol("do".into(), 0));
          v.extend(vec![
            Token::Whitespace("\n".into(), 0),
            Token::Symbol("end".into(), 0),
          ]);
        }
        v
      },
      path,
      conf,
    )
  }

  fn substitute_macro_params(
    &mut self,
    body: &[Token],
    param_names: &[String],
    args: &[Vec<Token>],
    path: Option<String>,
    conf: Option<LuluConf>,
  ) -> Vec<Token> {
    let mut result = Vec::new();

    for token in body {
      match token {
        Token::MacroParam(param, _) => {
          if let Some(index) = param_names.iter().position(|p| p == param) {
            if index < args.len() {
              result.extend(args[index].clone());
            } else {
              if param.starts_with("_") {
                result.extend(Vec::new());
              } else {
                panic!("Not enough arguments for macro parameter: ${}", param);
              }
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

    self.process_macros(result, path, conf)
  }

  fn generate_code(&self, tokens: Vec<Token>) -> String {
    let mut result = String::new();
    let mut i = 0;
    // let mut passed: std::collections::HashSet<&Token> = std::collections::HashSet::new();
    let hooks: HashMap<&Token, String> = HashMap::new();
    let mut hooks_int: HashMap<usize, String> = HashMap::new();

    while i < tokens.len() {
      let token = &tokens[i];

      // if passed.contains(token) {
      //   i += 1;
      //   continue;
      // }

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
        Token::Identifier(name, _) if name == "in" => {
          check_token!(&tokens, i, 1, true, Token::Identifier(ident, _) if ident == "do" => {
            let mut idx = i + 1;

            while idx < tokens.len() && matches!(tokens[idx], Token::Whitespace(_, _)) {
              idx += 1;
            }

            idx += 1;

            i = idx;

            result.push_str("(function()");
            pass_until_block_end!(tokens, idx, 0);
            hooks_int.insert(idx, ")()".to_string());
          }, check_token!(&tokens, i, 1, true, Token::Identifier(ident, _) if ident == "if" => {
            let mut idx = i + 1;

            i = idx;

            result.push_str("(function()\n");
            pass_until_block_end!(tokens, idx, 1);
            hooks_int.insert(idx, "\nend)()".to_string());
          }, check_token!(&tokens, i, 1, true, Token::Identifier(ident, _) if ident == "local" => {
            let mut idx = i + 1;

            while idx < tokens.len() && matches!(tokens[idx], Token::Whitespace(_, _)) {
              idx += 1;
            }
            
            idx += 1;
            
            while idx < tokens.len() && matches!(tokens[idx], Token::Whitespace(_, _)) {
              idx += 1;
            }

            let name = tokens[idx].clone();
            let mut p = "nil".to_string();

            idx += 1;

            let mut j = idx;

            while j < tokens.len() && matches!(tokens[j], Token::Whitespace(_, _)) {
              j += 1;
            }

            while j < tokens.len() && matches!(&tokens[j], Token::Identifier(id, _) if id == "and") {
              j += 1;
              while j < tokens.len() && matches!(tokens[j], Token::Whitespace(_, _)) {
                j += 1;
              }
              if j < tokens.len() && matches!(&tokens[j], Token::Identifier(_, _)) {
                if p == "nil" {
                  p = "".to_string();
                }
                if p != "" {
                  p.push_str(",");
                }
                p.push_str(get_token_string(&tokens[j]).unwrap());
                j += 1;
                idx = j;

                while j < tokens.len() && matches!(tokens[j], Token::Whitespace(_, _)) {
                  j += 1;
                }
              } else {
                break;
              }
            }

            i = idx;

            result.push_str(&format!("local {} = namespace(ns_inherit_from({p}))(function(self)\n", get_token_string(&name).unwrap()));

            pass_until_block_end!(tokens, idx, 0);
            hooks_int.insert(idx, ")".to_string());
            
          }, result.push_str(name))));
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
        Token::Symbol(sym, _) if sym == "!" => {
          check_token!(&tokens, i, 1, false, Token::Symbol(sym2, _) if sym2 == "=" => {
            result.push('~');
          }, result.push_str(sym));
        }
        Token::Symbol(sym, _) if sym == "+" || sym == "-" || sym == "*" || sym == "/" => {
          check_token!(&tokens, i, 1, false, Token::Symbol(sym2, _) if sym2 == "=" => {
            let mut j = i - 1;
            while j > 0 && matches!(tokens[j], Token::Whitespace(_, _)) {
              j -= 1;
            }
            if j > 0 && matches!(&tokens[j], Token::Identifier(_, _)) {
              hooks_int.insert(i + 1, format!(" {} {}", get_token_string(&tokens[j]).unwrap(), sym));
            }
          }, result.push_str(sym));
        }
        Token::Symbol(sym, _) if sym == "&" => {
          check_token!(&tokens, i, 1, false, Token::String(current_token, _)=> {
            result.push_str(format!("ptr_of({:?})", current_token).as_str());
            i += 1;
          }, check_token!(&tokens, i, 1, false, Token::Number(current_token, _) => {
            result.push_str(format!("ptr_of({:?})", current_token).as_str());
            i += 1;
          }, check_token!(&tokens, i, 1, false, Token::Identifier(current_token, _) => {
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
          let mut j = i + 1;
          let mut paren_depth = 1;

          while j < tokens.len() && paren_depth > 0 {
            match tokens[j] {
              Token::LeftParen(_) => paren_depth += 1,
              Token::RightParen(_) => paren_depth -= 1,
              _ => {}
            }
            j += 1;
          }

          let args_end = j;
          let mut name = String::new();
          let mut parent = String::new();
          let mut decorators: Vec<(String, Vec<Token>)> = Vec::new();

          while j < tokens.len() && matches!(tokens[j], Token::Whitespace(_, _)) {
            j += 1;
          }

          while j + 1 < tokens.len() {
            match (&tokens[j], &tokens[j + 1]) {
              (Token::Symbol(sym, _), Token::Identifier(id, _)) if sym == "@" => {
                let decor_name = id.clone();
                j += 2;

                let mut decor_args = Vec::new();
                if j < tokens.len() && matches!(tokens[j], Token::LeftParen(_)) {
                  decor_args.push(Token::Whitespace("".to_string(), 1));
                  let mut depth = 1;
                  j += 1;

                  while j < tokens.len() && depth > 0 {
                    match &tokens[j] {
                      Token::LeftParen(_) | Token::LeftBrace(_) => depth += 1,
                      Token::RightParen(_) | Token::RightBrace(_) => depth -= 1,
                      _ => {}
                    }

                    if depth > 0 {
                      decor_args.push(tokens[j].clone());
                    }
                    j += 1;
                  }
                }

                decorators.insert(0, (decor_name, decor_args));
              }

              (Token::Whitespace(_, _), _) => j += 1,
              _ => break,
            }
          }

          if j < tokens.len() && matches!(&tokens[j], Token::Identifier(i, _) if i != "end") {
            if let Some(n) = get_token_string(&tokens[j]) {
              if n == "async" {
                decorators.push(("async".to_string(), Vec::new()))
              } else {
                name = format!(" {}", n.clone());
              }
            }
            j += 1;
          }

          if j < tokens.len() && matches!(&tokens[j], Token::Symbol(s, _) if s == ":") {
            if j < tokens.len() && matches!(&tokens[j + 1], Token::Identifier(i, _) if i != "end") {
              let parent_name = get_token_string(&tokens[j + 1]).unwrap();
              parent = name.clone();
              name = format!("{}:{}", name, parent_name);
              j += 2;
            }
          }

          while j < tokens.len() && matches!(tokens[j], Token::Whitespace(_, _)) {
            j += 1;
          }

          if j < tokens.len() {
            if let Token::Symbol(ref sym, _) = tokens[j] {
              if sym == "=>" {
                let args_tokens = tokens[i..args_end].to_vec();
                let mut decorated_args: Vec<(String, Vec<(String, Vec<Token>)>)> = Vec::new();
                let mut k = 0;
                let mut current_decorators: Vec<(String, Vec<Token>)> = Vec::new();
                let mut vararg = String::new();

                while k < args_tokens.len() {
                  match &args_tokens[k] {
                    Token::Symbol(sym, _) if sym == "@" && k + 1 < args_tokens.len() => {
                      if let Token::Identifier(param_decorator, _) = &args_tokens[k + 1] {
                        k += 2;

                        let mut decor_args = Vec::new();
                        if k < args_tokens.len() && matches!(args_tokens[k], Token::LeftParen(_)) {
                          let mut depth = 1;
                          k += 1;
                          while k < args_tokens.len() && depth > 0 {
                            match args_tokens[k] {
                              Token::LeftParen(_) | Token::LeftBrace(_) => depth += 1,
                              Token::RightParen(_) | Token::RightBrace(_) => depth -= 1,
                              _ => {}
                            }
                            if depth > 0 {
                              decor_args.push(args_tokens[k].clone());
                            }
                            k += 1;
                          }
                        }

                        while k < tokens.len() && matches!(args_tokens[k], Token::Whitespace(_, _))
                        {
                          k += 1;
                        }

                        current_decorators.push((param_decorator.clone(), decor_args));
                      }
                    }
                    Token::Identifier(param_name, _) => {
                      decorated_args.push((param_name.clone(), current_decorators.clone()));
                      current_decorators = Vec::new();
                      k += 1;
                    }
                    Token::Symbol(sym, _) if sym == "," => {
                      k += 1;
                    }
                    Token::Symbol(sym, _) if sym == "." => {
                      vararg.push('.');
                      k += 1;
                    }
                    Token::Whitespace(_, _) => k += 1,
                    _ => {
                      k += 1;
                    }
                  }
                }

                let mut args_str = String::new();
                for (idx, (param_name, _)) in decorated_args.iter().enumerate() {
                  args_str.push_str(param_name);
                  if idx != decorated_args.len() - 1 && vararg.is_empty() {
                    args_str.push_str(", ");
                  }
                }
                if !vararg.is_empty() {
                  args_str.push_str(&vararg);
                }

                if !parent.is_empty() && !name.is_empty() && decorators.len() > 0 {
                  if !args_str.is_empty() {
                    args_str.insert_str(0, ",");
                  }
                  args_str.insert_str(0, "self");
                }

                let mut prefix = String::new();

                if decorators.len() > 0 {
                  let mut k = j + 1;
                  pass_until_block_end!(tokens, k, 0);

                  let mut hook_close = String::new();
                  for (decor, args) in decorators.iter().rev() {
                    let args_str = self.generate_code(args.clone());
                    prefix = format!(
                      "{}{}({}{}",
                      decor,
                      if args.is_empty() {
                        "".to_string()
                      } else {
                        format!("({})", args_str)
                      },
                      if parent.is_empty() {
                        "".to_string()
                      } else {
                        format!("{},", parent)
                      },
                      prefix
                    );
                    if decor == "async" {
                      hook_close.push(')');
                    } else {
                      hook_close.push_str(&format!(", {:?})", name.trim()));
                    }
                  }

                  hooks_int.insert(k, hook_close);

                  if !name.is_empty() {
                    prefix = format!("{} = {}", name.replace(':', "."), prefix);
                    name = String::new();
                  }
                }

                let mut deco_str = String::new();
                for (_, (param_name, decors)) in decorated_args.iter().enumerate() {
                  for (decor_name, decor_args) in decors {
                    let decor_args_str = self.generate_code(decor_args.clone());
                    deco_str.push_str(&format!("\n{} = ", param_name));
                    deco_str.push_str(&format!("{}", decor_name));
                    if !decor_args_str.is_empty() {
                      deco_str.push_str(&format!("({})", decor_args_str));
                    }
                    deco_str.push_str(&format!(
                      "({}, {}, {:?})",
                      if parent.is_empty() {
                        "empty_class()"
                      } else {
                        "self"
                      },
                      param_name,
                      param_name
                    ));
                  }
                }

                let fn_code = format!("{}function{}({}){}", prefix, name, args_str, deco_str);

                result.push_str(&fn_code);

                i = j + 1;
                continue;
              }
            }
          }

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

      if let Some(hook) = hooks.get(&tokens[i]) {
        result.push_str(hook);
      }

      if let Some(hook) = hooks_int.get(&i) {
        result.push_str(hook);
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
