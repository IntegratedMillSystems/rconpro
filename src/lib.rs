pub mod sockets;
use sockets::{SetupStream, CPSocket};
pub mod eip;

pub struct Service {
  pub plcs: Vec<Plc>,
  pub cpsocket: CPSocket
}

pub struct Plc {
  addr: String,
  setupstream: SetupStream,
  consumers: Vec<Consumer>,
}

pub struct ConsumerHint {
  pub tag: String,
  pub data_size: usize,
  pub rpi: usize,
  pub otrpi: usize,
}

pub struct Consumer {
  plc: Plc,
  hint: ConsumerHint,
  handler: fn(&[u8]),

  ot_connection_id: u32,
  sequence_count: u32,
}


impl Service {
  pub fn new() -> std::io::Result<Service> {
    Ok(Service {
      cpsocket: CPSocket::new()?,
      plcs: Vec::new(),
    })
  }

  pub fn addPlc(&mut self, addr: String) -> std::io::Result<()> {
    self.plcs.push(Plc::new(addr)?);
    Ok(())
  }
}

impl Plc {
  pub fn new(addr: String) -> std::io::Result<Plc> {
    Ok(Plc {
      addr: (&addr).to_string(),
      setupstream: SetupStream::new(&addr)?,
      consumers: Vec::new(),
    })
  }
}