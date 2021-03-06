use std::io::{Result, Cursor};
use std::collections::HashMap;
use std::sync::Arc;
use byteorder::{ReadBytesExt, LittleEndian};

use crate::sockets::{EipAddr, SetupStream};
use crate::eip::build_register_session;
use crate::{Consumer, ConsumerHint, ConsumerQueue};

/*
The struct representing PLCs
All consumers are owned by a Plc struct
*/
pub(crate) struct Plc {
  pub(crate) addr: EipAddr,
  pub(crate) consumers: HashMap<u32, Consumer>,
  pub(crate) setup_stream: SetupStream,
  pub(crate) session_handle: u32
}

impl Plc {
  /*
  Start socket and init Plc
  */
  pub(crate) fn new(addr: EipAddr) -> std::io::Result<Plc> {
    Ok(Plc {
      addr: addr,
      consumers: HashMap::new(),
      setup_stream: SetupStream::new(),
      session_handle: 0
    })
  }

  pub(crate) fn connect(&mut self) -> std::io::Result<()> {
    self.setup_stream.connect(&self.addr)?;
    
    Ok(())
  }

  /*
  Register a connection with the actual PLC
  */
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
  
  /*
  Start a consumer and add it to the hashmap
  */
  pub(crate) fn add_consumer(&mut self, hint: ConsumerHint, queue: &Arc<ConsumerQueue>) -> (&Consumer, u32) {
    let mut con = Consumer::new(hint, queue);
    let to_connection_id = con.send_forward_open(&mut self.setup_stream, self.session_handle, self.addr.slot)
      .unwrap();

    self.consumers.insert(
      to_connection_id,
      con
    );

    (&self.consumers[&to_connection_id], to_connection_id)
  }
}