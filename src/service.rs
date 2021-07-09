use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicU32};
use std::sync::{Arc, RwLock, Mutex};
use std::io::{Result, Cursor, ErrorKind};
use std::collections::HashMap;
use byteorder::{ReadBytesExt, LittleEndian};

use crate::sockets::{EipAddr, CPSocket};
use crate::{ConsumerHint, Plc};

/*
Entrypoint of rconpro
Manages PLCs, Consumers, and incomming Consumer/Producer packets.
*/
pub struct Service{ 
  pub(crate) plcs: Arc<RwLock<HashMap<EipAddr, Plc>>>,
  pub(crate) cpsocket: Arc<Mutex<CPSocket>>,
  pub(crate) sequence_count: Arc<AtomicU32>, 
}
impl Service {

  /*
  Initiate the service
  This returns a result in case the binding of the CPSocket fails; make sure it
  is unwrapped.
  */
  pub fn new() -> Result<Service> {
    // Define
    let timeout = Duration::new(1,0);
    let service: Service = Service {
      plcs: Arc::new(RwLock::new(HashMap::new())),
      cpsocket: Arc::new(Mutex::new(CPSocket::new(timeout)?)),
      sequence_count: Arc::new(AtomicU32::new(0)),
    };

    Ok(service)
  }

  /*
  Adds a consumer
  This function calls all of the logic required to add and start a Consumer, regardless
  of whether or not a connection has already been made with the target PLC.
  */
  pub fn add_consumer(&mut self, addr: EipAddr, hint: ConsumerHint, handler: fn(&[u8])) -> Result<()> {
    // Get lock on plcs list
    let mut plcs = self.plcs.write()
      .expect("PLC HashMap Lock is poisened");

    // Get PLC
    let plc = if plcs.contains_key(&addr) {
      plcs.get_mut(&addr).unwrap()
    } else {
      let mut plc = Plc::new(addr)?;
      plc.register()?;

      plcs.insert(plc.addr, plc);
      plcs.get_mut(&addr).unwrap()
    };

    // Create consumer
    let con = plc.add_consumer(hint, handler);
    
    // Start keep alive response thread
    con.start_response_thread(&self.cpsocket, addr, &self.sequence_count);

    Ok(())
  }

  /*
  Starts the producer listening thread for the service
  */
  pub fn start(&self) {
    // Create locks for this thread
    let plcs_lock = Arc::clone(&self.plcs);
    let cpsocket_lock = Arc::clone(&self.cpsocket);

    /*
    Listens to producers, parses incomming data, and sends parsed data to the
    appropriate consumer handler.
    */
    thread::Builder::new().name("Listening Thread".to_string()).spawn(move || loop {

      // Sleep
      thread::sleep(Duration::new(0, 1));

      // Aquire socket Lock
      let mut cpsocket = cpsocket_lock.lock().unwrap();

      // Recieve data
      match cpsocket.recieve() {
        Ok((d, src_addr)) => {

          // Get id
          let mut cursor = Cursor::new(&d);
          cursor.set_position(6);
          let connection_id = cursor.read_u32::<LittleEndian>().unwrap();

          // Send data to appropriate consumer
          let plcs = plcs_lock.read().unwrap();
          if plcs.contains_key(&src_addr) {
            let plc = &plcs[&src_addr];

            if plc.consumers.contains_key(&connection_id) {
              (plc.consumers[&connection_id].handler)(&d[20..]);
              continue;
            }
          }

          // No consumer was found
          // This should happen.
          eprintln!("That was weird. We didn't find an active consumer for the data we recieved")
        },
        Err(e) => {
          if e.kind() == ErrorKind::WouldBlock {
            // The socket timed out
          } else {
            println!("{:}", e);
          }
        }
      }
    }).unwrap();
  }
}