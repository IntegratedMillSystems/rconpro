use std::net::{TcpStream, UdpSocket};
use std::io::{Read, Write, Error};
use std::str::from_utf8;

const SETUP_PORT: u16 = 44818;
const CONPRO_PORT: u16 = 2222;

const HEADER_SIZE: u8 = 24;

const BUF_SIZE: usize = 4096;

pub struct SetupStream {
  stream: TcpStream,
}

impl SetupStream {
  pub fn new(host: &str) -> Result<SetupStream, Error> {
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

  pub fn send_recieve(&mut self, msg: &[u8]) -> Result<String, Error> {
    self.stream.write(msg)
      .expect("Couldn't send data");
    
    let mut response = String::from("");
    let mut buf = [0 as u8; BUF_SIZE];
    loop {
      let size = self.stream.read(&mut buf)?;

      // Parse data
      let data_str = from_utf8( &buf[0..size] ) // Pass the part of the buf that has data
        .expect("Response is not UTF8!");
      response.push_str(data_str);
      
      if size == 0 || size < buf.len() {
        break;
      } // Else the stream needs to read again
    }

    Ok(response)
  }
}


pub struct CPSocket {
  socket: UdpSocket,
}

impl CPSocket {
  pub fn new() -> Result<CPSocket, Error> {
    Ok(CPSocket {
      socket: UdpSocket::bind(("0.0.0.0", CONPRO_PORT))?
    })
  }

  pub fn send(&mut self, msg: &[u8], ) -> Result<(), Error> {
    self.socket.send(msg)?;
    Ok(())
  }

  pub fn recieve(&mut self) -> Result<String, Error> {
    let mut response = String::from("");
    let mut buf = [0 as u8; BUF_SIZE];
    loop {
      let size = self.socket.recv(&mut buf)?;

      // Parse data
      let data_str = from_utf8( &buf[0..size] ) // Pass the part of the buf that has data
        .expect("Response is not UTF8!");
      response.push_str(data_str);
      
      if size == 0 || size < buf.len() {
        break;
      } // Else the stream needs to read again
    }

    Ok(response)
  }
}