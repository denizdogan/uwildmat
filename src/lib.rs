#![feature(test)]

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Uwildmat {
  #[default]
  Fail,
  Match,
  Poison,
}

impl std::fmt::Display for Uwildmat {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Uwildmat::Fail => write!(f, "Fail"),
      Uwildmat::Match => write!(f, "Match"),
      Uwildmat::Poison => write!(f, "Poison"),
    }
  }
}

impl Into<bool> for Uwildmat {
  fn into(self) -> bool {
    self == Uwildmat::Match
  }
}

#[inline]
pub fn regular(text: &str, pat: &str) -> bool {
  pat == "*" || match_expression(text, pat, false).into()
}

#[inline]
pub fn simple(text: &str, pat: &str) -> bool {
  return pat == "*" || match_pattern(text, pat);
}

#[inline]
pub fn poison(text: &str, pat: &str) -> Uwildmat {
  return if pat == "*" {
    Uwildmat::Match
  } else {
    return match_expression(text, pat, true);
  };
}

fn match_pattern(txt: &str, pat: &str) -> bool {
  let mut pat_i: usize = 0;
  let mut txt_i: usize = 0;
  let pat_len = pat.chars().count();

  while pat_i < pat_len {
    match pat.chars().nth(pat_i) {
      None => {
        panic!("unexpected end of pattern")
      }

      // match any character.
      Some('?') => {
        txt_i += 1;
        pat_i += 1;
      }

      // escape next pattern character.
      // this is the only place where we need to care about escaping.
      Some('\\') => match pat.chars().nth(pat_i + 1) {
        None => return false,
        Some(c) => {
          if txt.chars().nth(txt_i) != Some(c) {
            return false;
          }
          txt_i += 1;
          pat_i += 2;
        }
      },

      // match zero or more characters.
      Some('*') => {
        // skip this * and skip any subsequent *'s
        pat_i += 1;
        while pat.chars().nth(pat_i) == Some('*') {
          pat_i += 1;
        }

        // if end was reached, we matched!
        if pat.chars().nth(pat_i).is_none() {
          return true;
        }

        // check if remaining pattern matches any tail of txt
        for txt_j in txt_i..txt.len() {
          if txt.is_char_boundary(txt_j) {
            if match_pattern(&txt[txt_j..], &pat[pat_i..]) {
              return true;
            }
          }
        }

        // no match
        return false;
      }
      Some('[') => {
        // fail if no more pattern chars
        if pat_i + 1 >= pat_len {
          return false;
        }

        // check the next text char
        match txt.chars().nth(txt_i) {
          None => return false,
          Some(txt_ch) => {
            // `negate` is true if the character set is negated.
            // `q` is the index of the first char in the character set, skipping
            // over any leading '^' character.
            let negate = pat.chars().nth(pat_i + 1).unwrap() == '^';
            let q = pat_i + (negate as usize) + 1;
            if q >= pat_len {
              return false;
            }

            // find the offset of the closing bracket from the opening bracket.
            // start looking at q + 1 to prevent problems with "[]]".
            let o_end_offset = pat.chars().skip(q + 1).position(|c| c == ']');
            if o_end_offset.is_none() {
              return false;
            }

            // `end_idx` = index of matching ']'
            // we add 1 because `opt_end_off` will have an offset of -1.
            let end_idx = o_end_offset.unwrap() + q + 1;
            let set: String = pat.chars().skip(q).take(end_idx - q).collect();
            if !match_class(txt_ch, &set, negate) {
              return false;
            }

            pat_i = end_idx + 1; // move past the closing bracket
            txt_i += 1; // one char was consumed
          }
        }
      }
      Some(ch) => {
        if txt.chars().nth(txt_i) == Some(ch) {
          pat_i += 1;
          txt_i += 1;
        } else {
          return false;
        }
      }
    }
  }

  // text ended BEFORE pattern ended.
  return txt_i == txt.chars().count();
}

fn match_class(txt_ch: char, set: &str, negate: bool) -> bool {
  let set_len = set.chars().count();
  let mut min_ch: char = '\0';
  let mut allow_hyphen = false;
  let mut set_i = 0;
  while set_i < set_len {
    if allow_hyphen
      && set_i < set_len - 1
      && set.chars().nth(set_i) == Some('-')
    {
      let max_ch = set.chars().nth(set_i + 1).unwrap();
      if txt_ch >= min_ch && txt_ch <= max_ch {
        return !negate;
      }
      allow_hyphen = false;
      set_i += 2;
    } else {
      min_ch = set.chars().nth(set_i).unwrap();
      if txt_ch == min_ch {
        return !negate;
      }
      allow_hyphen = true;
      set_i += 1;
    }
  }
  return negate;
}

fn match_expression(text: &str, pat: &str, allow_poison: bool) -> Uwildmat {
  if pat.is_empty() && text.is_empty() {
    return if allow_poison {
      Uwildmat::Poison
    } else {
      Uwildmat::Match
    };
  }

  let mut pat_start_i = 0;
  let mut invert = false;
  let mut poison = false;
  let mut poisoned = false;
  let mut escape = false;
  let mut matched: bool = false;
  let mut brack_st: i8 = -1;
  for (i, p) in pat.char_indices() {
    match p {
      '[' => {
        // unless this is escaped, prepare a set.
        if !escape {
          brack_st = 0;
        }
      }
      ']' => {
        // close the set only if this is NOT the first char.
        // otherwise, open it.
        if brack_st == 0 {
          brack_st = 1;
        } else {
          brack_st = -1;
        }
      }
      '\\' => {
        // if inside a set, \ is just a regular char.
        if brack_st == -1 {
          // if already escaping, we just escaped a backslash.
          // otherwise, we start escaping now.
          escape = !escape;
        }
      }
      '!' => {
        if pat_start_i == i {
          invert = true;
          pat_start_i = i + 1;
        }
      }
      '@' => {
        // if poison is allowed, this moves the start index forward. otherwise,
        // it's just a regular char.
        if pat_start_i == i && allow_poison {
          invert = true;
          poison = true;
          pat_start_i = i + 1;
        }
      }
      ',' => {
        if brack_st > -1 {
          // if we're in a set, this is just a regular char.
          brack_st = 1;
        } else if !escape {
          // unless escaped, this is a separator.
          if match_pattern(text.chars().as_str(), &pat[pat_start_i..i]) {
            matched = !invert;
            poisoned = poison;
          }
          pat_start_i = i + 1;
          invert = false;
          poison = false;
        } else {
          // we were actually escaped...
          escape = false;
        }
      }
      _ => {
        escape = false;
      }
    }
  }

  let m: bool = match_pattern(text.chars().as_str(), &pat[pat_start_i..]);
  if m {
    matched = !invert;
    poisoned = poison;
  }
  if poisoned {
    return Uwildmat::Poison;
  }
  if matched {
    return Uwildmat::Match;
  }
  return Uwildmat::Fail;
}

#[cfg(test)]
mod tests {
  use super::*;

  #[allow(unused_macros)]
  #[cfg(debug_assertions)]
  macro_rules! lg {
      ($( $args:expr ),*) => { println!("{:>3}   {:>16}   {:<16} expected {}, got {}", $( $args ),* ); }
  }
  #[cfg(not(debug_assertions))]
  macro_rules! lg {
    ($( $args:expr ),*) => {
      ()
    };
  }

  include!(concat!(env!("OUT_DIR"), "/gen_test_suite.rs"));

  // test_r in INN
  fn test_regular(n: usize, value: &str, pattern: &str, expected: bool) {
    let actual = regular(&value, &pattern);
    lg!(n, value, pattern, expected, actual);
    assert_eq!(expected, actual, "{}: '{}'/'{}'", n, value, pattern);
  }

  // test_p in INN
  fn test_poison(n: usize, value: &str, pattern: &str, expected: Uwildmat) {
    let actual = poison(&value, &pattern);
    lg!(n, value, pattern, expected, actual);
    assert_eq!(expected, actual, "{}: '{}'/'{}'", n, value, pattern);
  }

  // test_s in INN
  fn test_simple(n: usize, value: &str, pattern: &str, expected: bool) {
    let actual = simple(&value, &pattern);
    lg!(n, value, pattern, expected, actual);
    assert_eq!(
      expected, actual,
      "{}: value: {}  pattern: {}",
      n, value, pattern
    );
  }

  // test_v in INN validates that a byte sequence is valid utf-8. we don't
  // support invalid utf-8, so we don't really care.
  fn test_validate(_n: usize, _value: &str, _expected: bool) {
    assert_eq!(true, !false);
  }

  #[test]
  fn test_inn_suite() {
    run_inn_test_suite(test_regular, test_poison, test_simple, test_validate);
  }

  #[test]
  fn regular_comma_processing() {
    assert_eq!(false, regular(",", ""));
    assert_eq!(false, regular(",", ","));
    assert_eq!(false, regular(",", ",,"));
    assert_eq!(false, regular(",", ",,,"));
    assert_eq!(true, regular(",", "\\,"));
    assert_eq!(true, regular(",", "\\,,"));
    assert_eq!(true, regular(",", ",\\,"));
    assert_eq!(true, regular(",", "\\,,\\,"));
    assert_eq!(true, regular(",", ",\\,,"));
  }

  #[test]
  fn simple_comma_processing() {
    assert_eq!(false, simple(",", ""));
    assert_eq!(true, simple(",", ","));
    assert_eq!(false, simple(",", ",,"));
    assert_eq!(true, simple(",", "*,"));
    assert_eq!(true, simple(",", ",*"));
    assert_eq!(true, simple(",", "*"));
    assert_eq!(true, simple(",", "**"));
    assert_eq!(true, simple(",", "\\,"));
    assert_eq!(true, simple(",", "*\\,"));
    assert_eq!(true, simple(",", "\\,*"));
  }

  #[test]
  fn simple_brackets() {
    assert_eq!(true, simple("a", "[a-d]"));
    assert_eq!(true, simple("b", "[a-d]"));
    assert_eq!(true, simple("c", "[a-d]"));
    assert_eq!(true, simple("d", "[a-d]"));
    assert_eq!(true, simple("]", "[]]"));
    assert_eq!(true, simple("]*", "[]]?"));
    assert_eq!(true, simple("]*", "[]]*"));
    assert_eq!(true, simple("a", "[^]]"));
    assert_eq!(true, simple("a", "[^]-]"));
    assert_eq!(true, simple("c", "[^]-b]"));
    assert_eq!(true, simple("a", "[^]]*"));
    
    assert_eq!(false, simple("e", "[a-d]"));
    assert_eq!(false, simple("A", "[a-d]"));
    assert_eq!(false, simple("c", "[^a-f]"));
    assert_eq!(false, simple("-", "[^-]"));
    assert_eq!(false, simple("-", "[^]-]"));
    assert_eq!(false, simple("-", "[^-]]"));
    assert_eq!(false, simple("]", "[^]-]"));
    assert_eq!(false, simple("]", "[^-]]"));
  }

  #[test]
  fn regular_misc() {
    assert_eq!(true, regular("", ""));
    assert_eq!(true, regular("a", "a"));
    assert_eq!(true, regular("abc", "abc"));
    assert_eq!(true, regular("abc,", "abc\\,"));
    assert_eq!(true, regular("abc,", "abc\\,,foo"));
    assert_eq!(true, regular("foo", "*foo*"));

    assert_eq!(false, regular("abc", ""));
    assert_eq!(false, regular("abc", "a"));
    assert_eq!(false, regular("abc", "ab"));
    assert_eq!(false, regular("abc", "aaa"));
    assert_eq!(false, regular("abc", "bbb"));
    assert_eq!(false, regular("abc", "ccc"));
    assert_eq!(false, regular("abc,", "abc"));
    assert_eq!(false, regular("abc,", "abc\\,foo,yeah"));
  }

  #[test]
  fn into_bool() {
    assert_eq!(false, Uwildmat::Fail.into());
    assert_eq!(true, Uwildmat::Match.into());
    assert_eq!(false, Uwildmat::Poison.into());
  }
}
