use std::io::{Result, Cursor};
use byteorder::{ReadBytesExt, LittleEndian};

use crate::sockets::SetupStream;
use crate::eip::build_register_session;

pub struct Plc {
  pub(crate) addr: String,
  active_consumer_count: usize,
  setup_stream: SetupStream,
  session_handle: u32
}

impl Plc {
  pub fn new(addr: String) -> std::io::Result<Plc> {
    Ok(Plc {
      addr: (&addr).to_string(),
      active_consumer_count: 0,
      setup_stream: SetupStream::new(&addr)?,
      session_handle: 0
    })
  }

  pub fn register(&mut self) -> Result<()> {
    let reg_response = self.setup_stream.send_recieve(
      &build_register_session().as_slice()
    )?;

    if reg_response.len() > 0 {
      let mut cursor = Cursor::new(reg_response);
      cursor.set_position(4);
      self.session_handle = cursor.read_u32::<LittleEndian>().unwrap();
    } else {
      // TODO: throw error...
    }

    Ok(())
  }

  pub fn get_active_consumer_count(&self) -> usize {
    return self.active_consumer_count;
  }

  pub(crate) fn increment_active_consumer_count(&mut self) {
    self.active_consumer_count += 1;
  }
}