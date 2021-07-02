use std::{thread, time::Duration};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock, Mutex};
use std::io::{Result, Cursor, ErrorKind};
use std::collections::HashMap;

use byteorder::{ReadBytesExt, LittleEndian};

use crate::sockets::CPSocket;
use crate::{eip, Consumer};

pub struct Service {
  pub(crate) consumers: Arc<RwLock<HashMap<u32, Consumer>>>,
  pub(crate) cpsocket: Arc<Mutex<CPSocket>>,
  pub(crate) sequence_count: Arc<AtomicU32>, 
}

impl Service {
  pub fn new() -> Result<Service> {
    // Define
    let timeout = Duration::new(1,0);
    let service: Service = Service {
      consumers: Arc::new(RwLock::new(HashMap::new())),
      cpsocket: Arc::new(Mutex::new(CPSocket::new(timeout)?)),
      sequence_count: Arc::new(AtomicU32::new(0)),
    };

    Ok(service)
  }

  pub fn add_consumer(&mut self, con: Consumer) {
    con.start_response_thread(&self.cpsocket, &self.sequence_count);

    self.consumers.write().unwrap().insert(
      con.to_connection_id,
      con
    );
  }

  pub fn start(&self) {
    let consumers_lock = Arc::clone(&self.consumers);
    let cpsocket_lock = Arc::clone(&self.cpsocket);

    thread::Builder::new().name("Listening Thread".to_string()).spawn(move || loop {

      // Sleep
      thread::sleep(Duration::new(0, 1));

      // Lock
      let mut cpsocket = cpsocket_lock.lock().unwrap();

      // Recieve data
      match cpsocket.recieve() {
        Ok(d) => {
          // Get id
          let mut cursor = Cursor::new(&d);
          cursor.set_position(6);
          let connection_id = cursor.read_u32::<LittleEndian>().unwrap();

          // Send data to appropriate consumer
          let consumers = consumers_lock.read().unwrap();
          if consumers.contains_key(&connection_id) {
            (consumers[&connection_id].handler)(&d[20..]);
          } else {
            // TODO: Throw error
          }
        },
        Err(e) => {
          if e.kind() == ErrorKind::WouldBlock {
            println!("Timed Out");
          } else {
            println!("{:}", e);
          }
        }
      }
    }).unwrap();
  }
}