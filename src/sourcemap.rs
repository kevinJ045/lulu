use std::cmp::min;

#[derive(Debug, Clone)]
pub struct SourceMapEntry {
  pub lulu_line: usize,
  #[allow(unused)]
  pub lulu_col: usize,
  pub lua_line: usize,
  #[allow(unused)]
  pub lua_col: usize,
}

#[allow(unused)]
pub fn generate_sourcemap(lulu_source: &str, lua_output: &str) -> Vec<SourceMapEntry> {
  let lulu_chars: Vec<(usize, usize, char)> = lulu_source
    .lines()
    .enumerate()
    .flat_map(|(line_idx, line)| {
      line
        .chars()
        .enumerate()
        .map(move |(col_idx, c)| (line_idx, col_idx, c))
    })
    .collect();

  let lua_chars: Vec<(usize, usize, char)> = lua_output
    .lines()
    .enumerate()
    .flat_map(|(line_idx, line)| {
      line
        .chars()
        .enumerate()
        .map(move |(col_idx, c)| (line_idx, col_idx, c))
    })
    .collect();

  let mut dp = vec![vec![0; lua_chars.len() + 1]; lulu_chars.len() + 1];
  for i in 1..=lulu_chars.len() {
    for j in 1..=lua_chars.len() {
      if lulu_chars[i - 1].2 == lua_chars[j - 1].2 {
        dp[i][j] = dp[i - 1][j - 1] + 1;
      } else {
        dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
      }
    }
  }

  let mut sourcemap = Vec::new();
  let mut i = lulu_chars.len();
  let mut j = lua_chars.len();
  while i > 0 && j > 0 {
    if lulu_chars[i - 1].2 == lua_chars[j - 1].2 {
      sourcemap.push(SourceMapEntry {
        lulu_line: lulu_chars[i - 1].0,
        lulu_col: lulu_chars[i - 1].1,
        lua_line: lua_chars[j - 1].0,
        lua_col: lua_chars[j - 1].1,
      });
      i -= 1;
      j -= 1;
    } else if dp[i - 1][j] >= dp[i][j - 1] {
      i -= 1;
    } else {
      j -= 1;
    }
  }

  sourcemap.reverse();

  let mut filled = Vec::new();
  let mut last: Option<SourceMapEntry> = None;
  for entry in sourcemap {
    if let Some(prev) = last {
      let lua_gap = entry.lua_line.saturating_sub(prev.lua_line);
      let lulu_gap = entry.lulu_line.saturating_sub(prev.lulu_line);
      if lua_gap > 1 || lulu_gap > 1 {
        for i in 1..=lua_gap {
          filled.push(SourceMapEntry {
            lulu_line: prev.lulu_line + min(i, lulu_gap),
            lulu_col: 0,
            lua_line: prev.lua_line + i,
            lua_col: 0,
          });
        }
      }
    }
    filled.push(entry.clone());
    last = Some(entry);
  }

  filled
}

#[allow(unused)]
fn levenshtein(a: &str, b: &str) -> usize {
  let mut costs: Vec<usize> = (0..=b.len()).collect();
  for (i, ca) in a.chars().enumerate() {
    let mut last_cost = i;
    let mut new_costs = vec![i + 1];
    for (j, cb) in b.chars().enumerate() {
      let insertion = costs[j + 1] + 1;
      let deletion = new_costs[j] + 1;
      let substitution = if ca == cb { costs[j] } else { costs[j] + 1 };
      new_costs.push(*[insertion, deletion, substitution].iter().min().unwrap());
    }
    costs = new_costs;
  }
  *costs.last().unwrap()
}

#[allow(unused)]
pub fn lookup_lua_to_lulu(
  lua_line: usize,
  lua_col: usize,
  sourcemap: &[SourceMapEntry],
) -> Option<(usize, usize)> {
  sourcemap
    .iter()
    .filter(|entry| entry.lua_line == lua_line)
    .min_by_key(|entry| (entry.lua_col as isize - lua_col as isize).abs())
    .map(|entry| (entry.lulu_line, entry.lulu_col))
}

#[allow(unused)]
pub fn lookup_lulu_to_lua(
  lulu_line: usize,
  lulu_col: usize,
  sourcemap: &[SourceMapEntry],
) -> Option<(usize, usize)> {
  sourcemap
    .iter()
    .filter(|entry| entry.lulu_line == lulu_line)
    .min_by_key(|entry| (entry.lulu_col as isize - lulu_col as isize).abs())
    .map(|entry| (entry.lua_line, entry.lua_col))
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::compiler::Compiler;

  #[test]
  fn test_sourcemap() {
    let mut compiler = Compiler::new(None);

    let lulu = r#"r#"
a = 1
b = 2


c = 4


macro {
    hello($dd) { $dd }
}

local name = hello! "ddd";

func(() =>
    print('hi')
end)
"#;
    let lua = compiler.compile(lulu, None, None);

    let map = generate_sourcemap(lulu, &lua);
    for entry in map {
      println!(
        "Lua [{}:{}] -> Lulu [{}:{}]",
        entry.lua_line, entry.lua_col, entry.lulu_line, entry.lulu_col
      );
    }
  }
}
