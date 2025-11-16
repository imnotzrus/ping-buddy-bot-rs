//! Utility macros for common patterns.

/// Unwrap an Option, or return Ok(()) if None
///
/// This macro is useful for early returns in handlers when an optional
/// value is not present and we want to silently skip processing.
///
/// # Example
///
/// ```ignore
/// let user = some_rtn_ok!(msg.from.as_ref());
/// // If msg.from is None, the function returns Ok(()) early
/// ```
#[macro_export]
macro_rules! some_rtn_ok {
  ($expr:expr) => {{
    let Some(val) = $expr else {
      return Ok(());
    };
    val
  }};
}

pub use some_rtn_ok;
