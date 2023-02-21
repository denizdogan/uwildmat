use crate::{match_expression, Uwildmat};

/// User-level routine used for wildmats where @ should be treated as a
/// regular character.
#[inline]
pub fn uwildmat(text: &str, pat: &str) -> bool {
  return pat == "*" || match_expression(text, pat, false) == Uwildmat::Match;
}
