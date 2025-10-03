#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  Number(i64, usize),
  Identifier(String, usize),
  String(String, usize),
  BraceString(String, usize),
  Symbol(String, usize),
  Whitespace(String, usize),
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
      } else if ch.is_ascii_alphabetic() || ch == '_' {
        return self.read_identifier();
      } else if ch == '"' || ch == '\'' {
        return self.read_string();
      } else if ch == '[' && self.chars.get(self.pos + 1) == Some(&'[') {
        return self.read_string();
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

  fn read_identifier(&mut self) -> Token {
    let mut s = String::new();
    while let Some(ch) = self.peek_char() {
      if ch.is_ascii_alphanumeric() || ch == '_' {
        s.push(ch);
        self.next_char();
      } else {
        break;
      }
    }
    Token::Identifier(s, self.tokens.clone())
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
      continue; // skip whitespace without counting
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
    | Token::Whitespace(_, i) => i,
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

pub fn compile(code: &str) -> String {
  let tokens = tokenize(code);
  let mut result = String::new();

  // println!("{:?}", tokens);

  let mut i = 0;
  while i < tokens.len() {
    let token = &tokens[i];

    match token {
      // Token::Identifier(name) if name == "ss" => {
      //   // Replace "ss" with "dd"
      //   result.push_str("dd");
      // }
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
      Token::Whitespace(ws, _) => {
        result.push_str(ws);
      }
      Token::EOF(_) => {}
    }

    i += 1;
  }

  // println!("{}", result);
  result
}
