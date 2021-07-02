use std::net::{TcpStream, UdpSocket};
use std::io::{Read, Write, Result, Cursor};
use std::time::Duration;
use byteorder::{ReadBytesExt, LittleEndian};

const SETUP_PORT: u16 = 44818;
const CONPRO_PORT: u16 = 2222;

const HEADER_SIZE: usize = 24;

const BUF_SIZE: usize = 4096;

pub struct SetupStream {
  stream: TcpStream,
}

impl SetupStream {
  pub fn new(host: &str) -> Result<SetupStream> {
    let addr = (host, SETUP_PORT);
    match TcpStream::connect(addr) {
      Ok(stream) => {
        println!("Successfully connected to {}", host);
        return Ok(SetupStream {
                    stream: stream
                  });
      },
      Err(e) => {
        eprintln!("Failed to connect: {}", e);
        return Err(e);
      }
    }
  }

  pub fn send_recieve(&mut self, msg: &[u8]) -> Result<Vec<u8>> {
    self.stream.write(msg)
      .expect("Couldn't send data");
  
    let mut response = vec![];
    let mut buf = [0 as u8; BUF_SIZE];

    // Get data
    let size = self.stream.read(&mut buf)?;
    response.extend_from_slice(&buf[0..size]);

    let mut cursor = Cursor::new(&response);
    cursor.set_position(2);
    let data_len: usize = cursor.read_u16::<LittleEndian>().unwrap().into();
    println!("Message size: {:}", data_len);

    while response.len() - HEADER_SIZE < data_len {
      let size = self.stream.read(&mut buf)?;
      response.extend_from_slice( &buf[0..size] );
    }

    Ok(response)
  }
}


pub struct CPSocket {
  socket: UdpSocket,
}

impl CPSocket {
  pub fn new(timeout: Duration) -> Result<CPSocket> {
    let mut socket = UdpSocket::bind(("0.0.0.0", CONPRO_PORT))?;
    socket.set_read_timeout(Some(timeout));

    Ok(CPSocket {
      socket: socket
    })
  }

  pub fn send_to(&mut self, msg: &[u8], addr: &str) -> Result<()> {
    self.socket.send_to(msg, (addr, 2222))?;
    Ok(())
  }

  pub fn recieve(&mut self) -> Result<Vec<u8>> {
    let mut response = vec![];
    let mut buf = [0 as u8; BUF_SIZE];
    
    // Get data
    let size = self.socket.recv(&mut buf)?;
    response.extend_from_slice(&buf[0..size]);
    
    Ok(response)
  }
}