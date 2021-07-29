use std::net::{TcpStream, UdpSocket, IpAddr, SocketAddr};
use std::io::{Read, Write, Result, Cursor};
use std::time::Duration;
use std::hash::Hash;
use std::cmp::Eq;
use byteorder::{ReadBytesExt, LittleEndian};
use serde::{Deserialize, Serialize};

// CIP/EIP protocol constants
const SETUP_PORT: u16 = 44818;
const CONPRO_PORT: u16 = 2222;

const HEADER_SIZE: usize = 24;

// Buf size (arbitrary)
const BUF_SIZE: usize = 4096;



/*
The EipAddr struct is a combination of the IpAddr struct (as you would expect)
and a slot number, which is/wil be used by the EIP protocol to determine which
slot of the rack to send requests to.

The derive statement below allows us to...
Debug:                  print its value
Hash, PartialEq, Eq:    use as a key for a hashmap
Copy, Clone:            automatically clone when needed
*/
#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone, Deserialize, Serialize)]
pub struct EipAddr {
  pub addr: IpAddr,
  pub slot: u8,
}



/*
The SetupStream struct contains a TcpStream and some methods that use that stream
to set up a consumer/producer connection
*/
pub(crate) struct SetupStream {
  stream: Option<TcpStream>,
}
impl SetupStream {

  /*
  Creates a new instance
  This actually returns a result because the connection could fail; make sure that
  result is unwrapped.
  */
  pub(crate) fn new() -> SetupStream {
    SetupStream {
      stream: None
    }
  }

  pub(crate) fn connect(&mut self, host: &EipAddr) -> Result<()> {
    // Try to connect
    let socket_addr = SocketAddr::new(host.addr, SETUP_PORT);
    self.stream = Some(TcpStream::connect(socket_addr)?);

    Ok(())
  }

  /*
  Send a msg to the host and get the reply
  This function hang until all data is recieved.
  */
  pub(crate) fn send_recieve(&mut self, msg: &[u8]) -> Result<Vec<u8>> {
    // Send the message
    self.stream.as_ref().unwrap().write(msg)
      .expect("Couldn't send data");
  
    // Get the response
    // Set up vars
    let mut response = vec![];
    let mut buf = [0 as u8; BUF_SIZE];

    // Get first data packet
    let size = self.stream.as_ref().unwrap().read(&mut buf)?;
    response.extend_from_slice(&buf[0..size]);

    // Parse with file cursor
    let mut cursor = Cursor::new(&response);

    // Get the size of the entire method (a field of the protocol)
    cursor.set_position(2);
    let data_len: usize = cursor.read_u16::<LittleEndian>().unwrap().into();
    println!("Message size: {:}", data_len);

    // Get the rest of the message
    while response.len() - HEADER_SIZE < data_len {
      let size = self.stream.as_ref().unwrap().read(&mut buf)?;
      response.extend_from_slice( &buf[0..size] );
    }

    Ok(response)
  }
}

/*
A struct for recieving producer data and sending keep alive packets
*/
pub struct CPSocket {
  socket: Option<UdpSocket>,
}
impl CPSocket {

  /*
  Initialize a CPSocket
  As with the SetupStream, this function returns a result because the socket bind
  could fail; make sure the result is unwrapped.
  */
  pub fn new() -> CPSocket {
    CPSocket {
      socket: None
    }
  }

  pub fn bind(&mut self, timeout: Duration) -> Result<()> {
    // Create, bind socket and set timeout
    self.socket = Some( UdpSocket::bind(("0.0.0.0", CONPRO_PORT))? );
    self.socket.as_ref().unwrap().set_read_timeout(Some(timeout)).unwrap();
    
    Ok(())
  }

  /*
  Send a packet directly to a client
  This is used to send keep-alive packets
  */
  pub fn send_to(&self, msg: &[u8], host: &EipAddr) -> Result<()> {
    self.socket.as_ref().unwrap().send_to(msg, SocketAddr::new(host.addr, 2222))?;
    Ok(())
  }

  /*
  Recieve data from (any) producer
  Listens for data; max data size is BUF_SIZE.
  Also, return the src EipAddr for sending to the right consumer
  */
  pub fn recieve(&mut self) -> Result<(Vec<u8>, EipAddr)> {
    let mut response = vec![];
    let mut buf = [0 as u8; BUF_SIZE];
    
    // Get data
    let (size, src) = self.socket.as_ref().unwrap().recv_from(&mut buf)?;
    response.extend_from_slice(&buf[0..size]);

    // Parse src
    Ok((response, EipAddr {
      addr: src.ip(),
      slot: 0
    }))
  }
}