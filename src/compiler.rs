use std::collections::HashMap;

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
    MacroRegistry {
      macros: HashMap::new(),
    }
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

fn peek_through(
  tokens: &[Token],
  current: usize,
  how_many: isize,
  skip_whitespace: bool,
) -> Option<&Token> {
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
    Some(&tokens[idx as usize])
  }
}

fn get_token_idx(tok: &Token) -> usize {
  match *tok {
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

#[derive(Debug, Clone)]
pub struct Compiler {
  macros: MacroRegistry,
}

impl Compiler {
  pub fn new() -> Self {
    Compiler {
      macros: MacroRegistry::new(),
    }
  }

  pub fn compile(&mut self, code: &str) -> String {
    let tokens = tokenize(code);
    let processed_tokens = self.process_macros(tokens);
    self.generate_code(processed_tokens)
  }

  fn process_macros(&mut self, tokens: Vec<Token>) -> Vec<Token> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
      match &tokens[i] {
        Token::Macro(_) => {
          i = self.parse_macro_definition(&tokens, i, &mut result);
        }
        Token::MacroCall(name, _) => {
          i = self.expand_macro_call(&tokens, i, &mut result, name.clone());
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
    &self,
    tokens: &[Token],
    start: usize,
    result: &mut Vec<Token>,
    macro_name: String,
  ) -> usize {
    let macro_def = match self.macros.get_macro(&macro_name) {
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
          if brace_count < 1 {
            current_arg.push(tokens[i].clone());
          }
          i += 1;
        }
        Token::RightBrace(_) => {
          brace_count -= 1;
          i += 1;

          if brace_count == 0 && paren_count == 0 {
            if i >= tokens.len() || !matches!(tokens[i], Token::Comma(_)) {
              break;
            }
          } else {
            current_arg.push(tokens[i].clone());
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
        _ => {
          current_arg.push(tokens[i].clone());
          i += 1;
        }
      }
    }

    if !current_arg.is_empty() {
      args.push(current_arg);
    }

    let expanded = self.substitute_macro_params(&macro_def.body, &macro_def.params, &args);
    result.extend(expanded);

    i
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
          check_token!(&tokens, i, 1, true, Token::String(current_token, _) => {
            result.push_str(format!("ptr_of({:?})", current_token).as_str());
            i += 1;
          }, result.push_str(sym));
        }
        Token::Symbol(sym, _) if sym == "*" => {
          check_token!(&tokens, i, 1, false, Token::Identifier(current_token, ind) => {
            check_token!(&tokens, *ind, 1, true, Token::Symbol(char, ind) if char == &'='.to_string() => {
              let next_token = peek_through(&tokens, *ind, 1, true).unwrap_or(&Token::EOF(0));
              result.push_str(format!("ptr_set({}, {})", current_token, match next_token {
                 Token::String(s, _) => format!("{:?}", s),
                 Token::Identifier(s, _) => format!("{}", s),
                 Token::Number(s, _) => format!("{}", s),
                 _ => panic!("You can only set a pointer to a preset value")
              }).as_str());
              i = get_token_idx(next_token)
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
  let mut compiler = Compiler::new();
  compiler.compile(code)
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_simple_macro() {
    let code = r#"macro {
  hello ($name) {
    print("Hello, " .. $name)
  }
}

hello! "World""#;

    let result = compile(code);
    println!("Input: {}", code);
    println!("Output: {}", result);

    // Should expand to: print("Hello, " .. "World")
    assert!(result.contains("print"));
    assert!(result.contains("Hello"));
    assert!(result.contains("World"));
  }

  #[test]
  fn test_macros_field() {
    let code = r#"
manifest = {
  name = "ss"
}    

mods = {
  main = "main.lua"
}

macros = {
  mymacro ($item) {
    print($item)
  },

  mymacro2 ($item) {
    print($item)
  }
}

include = {
  "@lib"
}
"#;

    let result = wrap_macros(code);
    println!("Output: {}", result);

    assert!(result.contains("macro {"));
  }

  #[test]
  fn test_complex_macro() {
    let code = r#"macro {
  some_complex_thing ($a, $b) {
    $a($b)
  }

}

some_complex_thing! { print }, { "hello" }"#;

    let result = compile(code);
    println!("Input: {}", code);
    println!("Output: {}", result);

    assert!(result.contains("print"));
    assert!(result.contains("\"hello\""));
  }

  #[test]
  fn test_for_each_macro() {
    let code = r#"local items = {0, 5, 10}

macro {
  for_each ($key, $iterator, $expr) {
    for $key in ipairs($iterator) do
      $expr
    end
  }
}

for_each! item, items, {
  print(item)
}"#;

    let result = compile(code);
    println!("Input: {}", code);
    println!("Output: {}", result);

    // Should expand the macro call
    assert!(result.contains("for item in ipairs"));
    assert!(result.contains("print(item)"));
    assert!(result.contains("end"));
  }

  #[test]
  fn test_basic_macro_expansion() {
    let code = r#"macro {
  greet ($name) {
    print("Hello, " .. $name)
  }
}

greet! "Lulu""#;

    let result = compile(code);
    println!("Input: {}", code);
    println!("Output: {}", result);

    // Should expand the macro
    assert!(result.contains("print"));
    assert!(result.contains("Hello"));
    assert!(result.contains("Lulu"));
  }
}
