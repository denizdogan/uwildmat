use crate::match_pattern;

/// User-level routine for simple expressions (neither , nor ! are special).
#[inline]
pub fn uwildmat(text: &str, pat: &str) -> bool {
  return pat == "*" || match_pattern(text, pat);
}
