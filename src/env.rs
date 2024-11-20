use std::env;

use crate::Result;

pub struct Env {
  pub inbound: String,
  pub outbound: String,
}

impl Env {
  pub fn init() -> Result<Self> {
    let inbound = env::var("INBOUND")?;
    let outbound = env::var("OUTBOUND")?;
    Ok(Self { inbound, outbound })
  }
}
