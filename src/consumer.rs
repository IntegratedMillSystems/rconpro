use crate::Plc;

pub struct ConsumerHint {
  pub tag: String,
  pub data_size: usize,
  pub rpi: usize,
  pub otrpi: usize,
}

pub struct Consumer {
  plc: Plc,
  hint: ConsumerHint,
  pub(crate) handler: fn(Vec<u8>),

  pub(crate) to_connection_id: u32,
}


impl Consumer {
  pub fn new(plc: Plc, hint: ConsumerHint, handler: fn(Vec<u8>)) -> Consumer {
    Consumer {
      plc: plc,
      hint: hint,
      handler: handler,
      to_connection_id: 0,
    }
  }

  pub(crate) fn get_plc_addr(&self) -> &str {
    return &self.plc.addr;
  }
}