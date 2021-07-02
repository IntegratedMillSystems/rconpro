use std::io::{Result, Cursor};
use byteorder::{ReadBytesExt, LittleEndian};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;
use std::convert::TryInto;

use crate::{Plc, eip, sockets::CPSocket};

pub struct ConsumerHint {
  pub tag: String,
  pub data_size: usize,
  pub rpi: usize,
  pub otrpi: usize,
}

pub struct Consumer {
  plc: Plc,
  hint: Arc<ConsumerHint>,
  pub(crate) handler: fn(&[u8]),

  pub(crate) ot_connection_id: u32,
  pub(crate) to_connection_id: u32,
}


impl Consumer {
  pub fn new(plc: Plc, hint: ConsumerHint, handler: fn(&[u8])) -> Consumer {
    Consumer {
      plc: plc,
      hint: Arc::new(hint),
      handler: handler,
      ot_connection_id: 0,
      to_connection_id: 0,
    }
  }

  pub(crate) fn get_plc_addr(&self) -> &str {
    return &self.plc.addr;
  }

  pub fn send_forward_open(&mut self) -> Result<()> {
    // Send forward open and get response
    let msg = eip::build_forward_open_packet(self.plc.session_handle, &self.hint);
    let response = self.plc.setup_stream.send_recieve(&msg.as_slice()).unwrap();

    // Check response
    if response.len() < 52 {
      panic!("Unable to parse response");
      // Reset socket...
    } else {

      // Parse response
      let mut cursor = Cursor::new(response);
      cursor.set_position(42);

      let sts = cursor.read_i8().unwrap();
      if sts == 0 {

        // Parse IDs
        cursor.set_position(44);
        self.ot_connection_id = cursor.read_u32::<LittleEndian>().unwrap();
        self.to_connection_id = cursor.read_u32::<LittleEndian>().unwrap();

      } else {
        panic!("Forward open failed");
      }
    }

    Ok(())
  }

  pub(crate) fn start_response_thread(&self, cpsocket: &Arc<Mutex<CPSocket>>, sequence_count: &Arc<AtomicU32>) {
    let cpsocket_lock = Arc::clone(cpsocket);
    let sequence_count_lock = Arc::clone(sequence_count);

    let hint = Arc::clone(&self.hint);
    let ot_connection_id = self.ot_connection_id;
    let plc_addr = (&self.plc.addr).to_string();

    let duration = Duration::new(
      (hint.otrpi / 1_000_000).try_into().unwrap(), 
      ((hint.otrpi % 1_000_000) * 1_000).try_into().unwrap()
    );

    thread::Builder::new().name(format!("Response thread for {}", hint.tag)).spawn(move || loop {

      // Sleep
      thread::sleep(duration);

      // Lock
      let mut cpsocket = cpsocket_lock.lock().unwrap();

      // Send keep alive packets
      let msg = eip::build_response_packet(
        ot_connection_id,
        sequence_count_lock.fetch_add(1, Ordering::SeqCst),
      );

      println!("Sending response packet...");
      cpsocket.send_to(msg.as_slice(), &plc_addr).unwrap();

    }).unwrap();
  }
}