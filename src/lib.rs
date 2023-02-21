#![feature(test)]
extern crate test;

pub mod poison;
pub mod regular;
pub mod simple;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub enum Uwildmat {
  #[default]
  Fail = 0,
  Match = 1,
  Poison = 2,
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
  use crate::poison::uwildmat as poison;
  use crate::regular::uwildmat as regular;
  use crate::simple::uwildmat as simple;

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

  fn test_validate(_n: usize, _value: &str, _expected: bool) {
    assert_eq!(true, !false);
  }

  #[test]
  fn test_inn_suite() {
    run_inn_test_suite(test_regular, test_poison, test_simple, test_validate);
  }

  #[test]
  fn arbitrary_regular_good() {
    fn r(v: &str, p: &str) {
      test_regular(1000, v, p, true);
    }
    r("", "");
    r("a", "a");
    r("abc", "abc");
    r("abc,", "abc\\,");
    r("abc,", "abc\\,,foo");
    r("foo", "*foo*");
  }

  #[test]
  fn arbitrary_regular_bad() {
    fn r(v: &str, p: &str) {
      test_regular(1000, v, p, false);
    }
    r("abc", "");
    r("abc", "a");
    r("abc", "ab");
    r("abc", "aaa");
    r("abc", "bbb");
    r("abc", "ccc");
    r("abc,", "abc");
    r("abc,", "abc\\,foo,yeah");
  }
}
