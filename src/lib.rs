use std::{fmt, str::Chars};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Uwildmat {
  #[default]
  Fail,
  Match,
  Poison,
}

impl fmt::Display for Uwildmat {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl From<bool> for Uwildmat {
  fn from(val: bool) -> Self {
    if val {
      Uwildmat::Match
    } else {
      Uwildmat::Fail
    }
  }
}

#[inline(always)]
pub fn regular(text: &str, pattern: &str) -> bool {
  pattern == "*" || match_expr(text, pattern, false).into()
}

#[inline(always)]
pub fn simple(text: &str, pattern: &str) -> bool {
  pattern == "*" || match_str(text, pattern)
}

#[inline(always)]
pub fn poison(text: &str, pattern: &str) -> Uwildmat {
  if pattern == "*" {
    Uwildmat::Match
  } else {
    match_expr(text, pattern, true)
  }
}

#[inline]
fn match_expr(text: &str, expr: &str, allow_poison: bool) -> Uwildmat {
  if expr.is_empty() {
    return text.is_empty().into();
  }

  let mut start = 0;
  let mut negate = false;
  let mut poison = false;
  let mut poisoned = false;
  let mut matched = false;
  let mut it = expr.char_indices();

  while let Some((idx, curr)) = it.next() {
    match curr {
      '[' => {
        it.by_ref().skip_while(|(_, c)| *c != ']').count();
      }
      '\\' => {
        it.next();
      }
      '!' if start == idx => {
        negate = true;
        start = idx + 1;
      }
      '@' if start == idx && allow_poison => {
        negate = true;
        poison = true;
        start = idx + 1;
      }
      ',' => {
        if match_str(text.chars().as_str(), &expr[start..idx]) {
          matched = !negate;
          poisoned = poison;
        }
        start = idx + 1;
        negate = false;
        poison = false;
      }
      _ => {}
    }
  }

  // match the rest of the expression.
  // (we will get here if there are no commas in the expression!)
  let rest = match_str(text.chars().as_str(), &expr[start..]);
  if rest {
    matched = !negate;
    poisoned = poison;
  }
  return if poisoned {
    Uwildmat::Poison
  } else if matched {
    Uwildmat::Match
  } else {
    Uwildmat::Fail
  };
}

#[inline]
fn match_str(text: &str, pattern: &str) -> bool {
  match_chars(&mut text.chars(), &mut pattern.chars())
}

#[inline]
fn match_chars(text: &mut Chars, pattern: &mut Chars) -> bool {
  while let Some(curr) = pattern.next() {
    match curr {
      '?' => {
        text.next();
      }

      '\\' => match pattern.next() {
        Some(c) if text.next() == Some(c) => {}
        _ => return false,
      },

      '*' => {
        // skip subsequent *'s.  if the rest of the pattern is empty, or if it
        // consists of *'s only, it matches anything.
        let next = pattern.skip_while(|p| *p == '*').next();
        if next.is_none() {
          return true;
        }

        // reconstruct the pattern by putting back `next` (which we just
        // consumed by checking for `None`).
        let rest = next.into_iter().chain(pattern).collect::<String>();

        // return if `text`, or any tail of `text`, matches.
        return text.clone().any(|_c| {
          if match_chars(&mut text.clone(), &mut rest.chars()) {
            return true;
          }
          text.next();
          return false;
        });
      }

      '[' => {
        // check the next text char
        match text.next() {
          None => return false,
          Some(want) => {
            let mut negate = false;
            let mut ended = false;
            let set: String = pattern
              .enumerate()
              .map(|(i, c)| {
                if i == 0 {
                  negate = c == '^';
                  return Ok(if negate { None } else { Some(c) });
                } else if i == 1 && negate || c != ']' {
                  return Ok(Some(c));
                } else {
                  ended = true;
                  return Err(());
                }
              })
              .take_while(Result::is_ok)
              .filter_map(Result::unwrap)
              .collect();
            // if `ended` is false, the set was never closed, i.e. malformed
            if !ended || !match_set(want, &set, negate) {
              return false;
            }
          }
        }
      }

      ch => {
        if text.next() != Some(ch) {
          return false;
        }
      }
    }
  }

  // check that the entire text was consumed
  return text.next().is_none();
}

#[inline]
fn match_set(want: char, set: &str, negate: bool) -> bool {
  let mut chars = set.chars();
  let mut min_char = None;
  while let Some(curr) = chars.next() {
    if curr == '-' {
      if let Some(min) = min_char {
        if let Some(max) = chars.next() {
          if (min..=max).contains(&want) {
            return !negate;
          } else {
            min_char = None;
            continue;
          }
        }
      }
    }
    if want == curr {
      return !negate;
    }
    min_char = Some(curr);
  }
  return negate;
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

  #[test]
  fn test_inn_suite() {
    run_inn_test_suite(test_regular, test_poison, test_simple);
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

    assert_eq!(false, regular("abc", ""));
    assert_eq!(false, regular("abc", "a"));
    assert_eq!(false, regular("abc", "ab"));
    assert_eq!(false, regular("abc", "aaa"));
    assert_eq!(false, regular("abc", "bbb"));
    assert_eq!(false, regular("abc", "ccc"));
    assert_eq!(false, regular("abc,", "abc"));
    assert_eq!(false, regular("abc,", "abc\\,foo,yeah"));

    assert_eq!(true, regular("", "*"));
    assert_eq!(true, regular("*", "*"));
    assert_eq!(true, regular("*", "\\*"));
    assert_eq!(true, regular("foo", "*"));
    assert_eq!(true, regular("foo", "foo*"));
    assert_eq!(true, regular("foo", "*foo"));
    assert_eq!(true, regular("foo", "*foo*"));
    assert_eq!(true, regular("foo*", "*\\*"));
    assert_eq!(true, regular("foo*", "*\\**"));
    assert_eq!(true, regular("foobar", "*foo*"));
    assert_eq!(true, regular("foobar", "foo*"));
    assert_eq!(true, regular("foobar", "*bar"));
    assert_eq!(true, regular("foobar", "*ooba*"));

    assert_eq!(true, regular("hello world", "hel*rld"));
    assert_eq!(true, regular("hello world", "[^]]ello*"));
    assert_eq!(true, regular("hello world", "[^]-]ello*"));
    assert_eq!(true, regular("hello world", "hell[^]-]*"));
  }

  #[test]
  fn poison_misc() {
    assert_eq!(Uwildmat::Match, poison("", ""));
    assert_eq!(Uwildmat::Fail, poison("", "a"));
  }

  #[test]
  fn regular_utf8() {
    assert_eq!(true, regular("†", "?")); // \xE2\x80\xA0
    assert_eq!(true, regular("ᚻ", "[\u{16BA}-\u{16BC}]")); // \xE1\x9A\xBB
    assert_eq!(true, regular("╳", "*")); // \xE2\x95\xB3
    assert_eq!(true, regular("ข้", "??")); // \xE0\xB8\x82\xE0\xB9\x89
  }

  #[test]
  fn into_bool() {
    assert_eq!(false, Uwildmat::Fail.into());
    assert_eq!(true, Uwildmat::Match.into());
    assert_eq!(false, Uwildmat::Poison.into());
  }
}
