use std::io::{Result, Error, ErrorKind, Cursor};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::convert::TryInto;
use crossbeam::queue::SegQueue;
use byteorder::{ReadBytesExt, LittleEndian};

use crate::eip;
use crate::sockets::{EipAddr, CPSocket, SetupStream};

/*
A struct specifying consumer parameters
*/
pub struct ConsumerHint {
  pub tag: String,
  pub data_size: usize,
  pub rpi: usize,
  pub otrpi: usize,
}

/*
Enum Type for the handler function
*/
pub type ConsumerQueue = SegQueue<Vec<u8>>;

/*
The consumer struct
Responsible for a single consumer
*/
pub(crate) struct Consumer {
  hint: Arc<ConsumerHint>,
  pub(crate) queue: Arc<ConsumerQueue>,

  pub(crate) ot_connection_id: u32,
  pub(crate) to_connection_id: u32,

  alive: Arc<AtomicBool>,
}
impl Consumer {

  /*
  Initiate a consumer
  */
  pub(crate) fn new(hint: ConsumerHint, queue: &Arc<ConsumerQueue>) -> Consumer {
    Consumer {
      hint: Arc::new(hint),
      queue: queue.clone(),
      ot_connection_id: 0,
      to_connection_id: 0,
      alive: Arc::new(AtomicBool::new(true))
    }
  }

  /*
  Send a forward open to the producer
  This effectively starts the consumer connection
  */
  pub(crate) fn send_forward_open(&mut self, setup_stream: &mut SetupStream, session_handle: u32) -> Result<u32> {
    // Send forward open and get response
    let msg = eip::build_forward_open_packet(session_handle, &self.hint);
    let response = setup_stream.send_recieve(&msg.as_slice()).unwrap();

    // Check response
    if response.len() < 52 {
      return Err(Error::new(ErrorKind::Other, "unable to parse response"));
      // Probably should reset the socket after this...
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
        return Err(Error::new(ErrorKind::Other, "Forward open failed"))
      }
    }

    Ok(self.to_connection_id)
  }

  /*
  The the response (keep-alive) thread
  This is called by the add_consumer implimented for the Service struct
  */
  pub(crate) fn start_response_thread(&self, cpsocket: &Arc<Mutex<CPSocket>>, plc_addr: EipAddr, sequence_count: &Arc<AtomicU32>) {
    // Get locks for the thread
    let alive = self.alive.clone();
    let cpsocket_lock = Arc::clone(cpsocket);
    let sequence_count_lock = Arc::clone(sequence_count);

    let hint = Arc::clone(&self.hint);
    let ot_connection_id = self.ot_connection_id;

    // Calculate the requested delay between packets
    let duration = Duration::new(
      (hint.otrpi / 1_000_000).try_into().unwrap(), 
      ((hint.otrpi % 1_000_000) * 1_000).try_into().unwrap()
    );

    /*
    Send response packates at an interval
    */
    thread::Builder::new().name(format!("Response thread for {}", hint.tag)).spawn(move || {
      while alive.load(Ordering::Relaxed) {

        // Sleep
        thread::sleep(duration);

        // Aquire socket lock
        let cpsocket = cpsocket_lock.lock().unwrap();

        // Send keep alive packets and increment sequence_count
        let msg = eip::build_response_packet(
          ot_connection_id,
          sequence_count_lock.fetch_add(1, Ordering::SeqCst),
        );

        // Send the packet
        cpsocket.send_to(msg.as_slice(), &plc_addr).unwrap();
      }
    }).unwrap();
  }

  pub(crate) fn stop(&mut self) {
    self.alive.store(false, Ordering::Release);
  }
}