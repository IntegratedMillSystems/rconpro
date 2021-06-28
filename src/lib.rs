pub mod sockets;

use sockets::{SetupStream, CPSocket};

struct Service {
  plcs: Vec<Plc>,
  cpsocket: CPSocket
}

struct Plc {
  addr: String,
  setupstream: SetupStream,
  consumers: Vec<Consumer>,
}

struct ConsumerHint {
  tag: String,
  datasize: usize,
  rpi: usize,
  otrpi: usize,
}

struct Consumer {
  plc: Plc,
  hint: ConsumerHint,
  handler: fn(&[u8]),
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