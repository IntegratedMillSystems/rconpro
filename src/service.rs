use std::{error::Error, thread, time::Duration};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock, Mutex};
use std::io::{Result, Cursor};
use std::collections::HashMap;

use byteorder::{ReadBytesExt, LittleEndian};

use crate::sockets::CPSocket;
use crate::{eip, Consumer};

pub struct Service {
  pub consumers: Arc<RwLock<HashMap<u32, Consumer>>>,
  pub cpsocket: Arc<Mutex<CPSocket>>,
}

impl Service {
  pub fn new() -> Result<Service> {
    // Define
    let service: Service = Service {
      consumers: Arc::new(RwLock::new(HashMap::new())),
      cpsocket: Arc::new(Mutex::new(CPSocket::new()?)),
    };

    Ok(service)
  }

  pub fn start(&self) {
    self.start_listening_thread();
    self.start_keep_alive_thread();
  }

  pub fn add_consumer(&mut self, con: Consumer) {
    self.consumers.write().unwrap().insert(
      con.to_connection_id,
      con
    );
  }

  fn start_listening_thread(&self) {
    let consumers_lock = Arc::clone(&self.consumers);
    let cpsocket_lock = Arc::clone(&self.cpsocket);

    thread::spawn(move || loop {
      let mut cpsocket = cpsocket_lock.lock().unwrap();
      match cpsocket.recieve() {
        Ok(d) => {
          // Get id
          let mut cursor = Cursor::new(&d);
          cursor.set_position(6);
          let connection_id = cursor.read_u32::<LittleEndian>().unwrap();

          // Send data to appropriate consumer
          let consumers = consumers_lock.read().unwrap();
          if consumers.contains_key(&connection_id) {
            (consumers[&connection_id].handler)(d);
          } else {
            // TODO: Throw error
          }
        },
        Err(e) => {

        }
      }

      thread::sleep(Duration::new(0, 1));
    });
  }

  fn start_keep_alive_thread(&self) {
    let consumers_lock = Arc::clone(&self.consumers);
    let cpsocket_lock = Arc::clone(&self.cpsocket);
    let mut sequence_count = 0;

    thread::spawn(move || loop {
      let mut cpsocket = cpsocket_lock.lock().unwrap();
      let consumers = consumers_lock.read().unwrap();
      
      for (connection_id, con) in consumers.iter() {
        let msg = eip::build_response_packet(*connection_id, sequence_count);
        sequence_count += 1;

        cpsocket.send_to(msg.as_slice(), con.get_plc_addr()).unwrap();
      }

      // Sleep
      thread::sleep(Duration::new(2, 0));
    });
  }
}