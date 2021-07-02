use rconpro::{Service, Plc, Consumer, ConsumerHint};

use std::io::{stdin, stdout, Read, Write};

fn handler(d: &[u8]) {
  println!("{:?}", d);
}

fn main() {
  let mut service = Service::new().unwrap();
  service.start();

  let mut plc = Plc::new("172.16.13.200").unwrap();
  plc.register().unwrap();
  
  let hint = ConsumerHint {
    tag: String::from("test"),
    data_size: 6,
    otrpi: 1_100_000,
    rpi: 1_000_000
  };

  let mut con = Consumer::new(plc, hint, handler);
  con.send_forward_open().unwrap();

  service.add_consumer(con);

  // Wait for enter
  let mut stdout = stdout();
  stdout.write(b"Press ENTER to close...").unwrap();
  stdout.flush().unwrap();
  stdin().read(&mut [0]).unwrap();
}