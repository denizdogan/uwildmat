use crate::{match_expression, Uwildmat};

/// User-level routine used for wildmats that support poison matches.
#[inline]
pub fn uwildmat(text: &str, pat: &str) -> Uwildmat {
  return if pat == "*" {
    Uwildmat::Match
  } else {
    return match_expression(text, pat, true);
  };
}
