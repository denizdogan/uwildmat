#![feature(iter_intersperse)]
use std::env;
use std::fs;
use std::path::Path;
use std::str;

use regex::Captures;
use regex::Regex;

const C_URL: &str = "https://raw.githubusercontent.com/InterNetNews/inn/5151f24ef885f7e18fc582c64b33fe10a1751636/tests/lib/uwildmat-t.c";
const RUST_FILENAME: &str = "gen_test_suite.rs";

#[tokio::main]
async fn main() {
  let out_dir = env::var_os("OUT_DIR").unwrap();
  let dest_path = Path::new(&out_dir).join(RUST_FILENAME);

  // get the c source code
  let c_code = fetch_code().await;

  // extract the test suite
  let c_tests: String = extract_suite(c_code);

  // put multiline statements on single lines
  let single_lines = c_tests.replace(",\n", ", ");

  // skip malformed utf-8 byte sequences
  let no_malformed = skip_malformed(single_lines);

  // replace C byte string sequences with from_utf8_unchecked calls
  let rustified = to_unchecked_calls(no_malformed);

  // generate rust source code
  let rust_code = make_rust_source(rustified);

  // write to file
  fs::write(&dest_path, rust_code).unwrap();
  println!("cargo:rerun-if-changed=src/build.rs");
}

async fn fetch_code() -> String {
  return reqwest::get(C_URL).await.unwrap().text().await.unwrap();
}

fn extract_suite(c_code: String) -> String {
  return c_code
    .split("/* clang-format off */")
    .nth(1)
    .unwrap()
    .split("/* clang-format on */")
    .nth(0)
    .into_iter()
    .collect();
}

// C doesn't have a concept of UTF-8, so we need to ignore some of the malformed
// UTF-8 byte sequences from the C source. one important difference between this
// rust implementation and its C predecessor is that this rust implementation
// _requires_ valid utf-8 strings, whereas the C version will accept any byte
// sequence, and if it cannot decode something as utf-8, it will fallback to
// just using the first octet.
fn skip_malformed(src: String) -> String {
  let skipped = [
    93, 99, 100, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
    116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 168, 169, 170, 171, 172,
    173, 174,
  ]
  .map(|n| format!("({:>3}, ", n));
  return src
    .lines()
    .map(|line| {
      let is_skipped = skipped.iter().any(|ig| line.contains(ig));
      if is_skipped {
        format!("// SKIPPED: // {}", line)
      } else {
        line.to_string()
      }
    })
    .map(|line| line.to_string() + "\n")
    .collect();
}

fn to_unchecked_calls(src: String) -> String {
  // "unescape" C string backslashes
  let code = src.replace(r"\\", r"\");

  // find C strings, group 1 is the content
  let str_rx = Regex::new(r#""(.*?)""#).unwrap();

  // find either a C octet or a literal char
  let oct_rx = Regex::new(r"\\(?P<oct>\d{1,3})|(?P<lit>.)").unwrap();

  // for each C string
  let fixed = str_rx.replace_all(&code, |st: &Captures| {
    let content = &st[1];
    // for each C octet or literal char,
    // represent it as a u8,
    // convert the u8 to a string,
    // then join the strings with ", ",
    // making e.g. "[97, 98, 99, 100]" from input "abc\100".
    let replaced = oct_rx
      .captures_iter(content)
      .map(|cap| {
        if let Some(oct) = cap.name("oct") {
          return u8::from_str_radix(oct.into(), 8).unwrap();
        } else if let Some(lit) = cap.name("lit") {
          return lit.as_str().as_bytes()[0];
        } else {
          panic!("should never happen, captures: '{:?}'", cap);
        }
      })
      .map(|b| b.to_string())
      .intersperse(", ".to_string());
    // wrap the result in a from_utf8_unchecked call
    return "from_utf8_unchecked(&[".to_string()
      + replaced.collect::<String>().as_str()
      + "])";
  });
  return fixed.to_string();
}

fn make_rust_source(code: String) -> String {
  format!(
    r#"
use std::str::from_utf8_unchecked;
const UWILDMAT_MATCH: Uwildmat = Uwildmat::Match;
const UWILDMAT_FAIL: Uwildmat = Uwildmat::Fail;
const UWILDMAT_POISON: Uwildmat = Uwildmat::Poison;
pub(crate) fn run_inn_test_suite(
  test_r: fn(usize, &str, &str, bool),
  test_p: fn(usize, &str, &str, Uwildmat),
  test_s: fn(usize, &str, &str, bool),
  test_v: fn(usize, &str, bool),
) {{
  unsafe {{
  {source}
}};
}}
"#,
    source = code
  )
  .trim()
  .to_string()
}