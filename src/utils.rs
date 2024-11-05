macro_rules! some_rtn_ok {
  ($expr:expr) => {{
    let Some(val) = $expr else {
      return Ok(());
    };
    val
  }};
}

pub(crate) use some_rtn_ok;
