use std::io::{Result, Cursor};
use std::collections::HashMap;
use byteorder::{ReadBytesExt, LittleEndian};

use crate::sockets::{EipAddr, SetupStream};
use crate::eip::build_register_session;
use crate::{Consumer, ConsumerHint};

/*

*/
pub(crate) struct Plc {
  pub(crate) addr: EipAddr,
  pub(crate) consumers: HashMap<u32, Consumer>,
  pub(crate) setup_stream: SetupStream,
  pub(crate) session_handle: u32
}

impl Plc {
  pub(crate) fn new(addr: EipAddr) -> std::io::Result<Plc> {
    Ok(Plc {
      addr: addr,
      consumers: HashMap::new(),
      setup_stream: SetupStream::new(&addr)?,
      session_handle: 0
    })
  }

  pub(crate) fn register(&mut self) -> Result<()> {
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
  
  pub(crate) fn add_consumer(&mut self, hint: ConsumerHint, handler: fn(&[u8])) -> &Consumer {
    let mut con = Consumer::new(hint, handler);
    let to_connection_id = con.send_forward_open(&mut self.setup_stream, self.session_handle)
      .unwrap();

    self.consumers.insert(
      to_connection_id,
      con
    );

    return &self.consumers[&to_connection_id];
  }
}