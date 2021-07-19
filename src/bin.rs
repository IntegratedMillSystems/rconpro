use rconpro::{Service, EipAddr, ConsumerHint};
use std::net::{IpAddr, Ipv4Addr};
use std::io::{stdin, stdout, Read, Write};

fn handler(d: &[u8]) {
  println!("{:?}", d);
}

fn main() {
  let mut service = Service::new().unwrap();
  service.start();

  let addr = EipAddr {
    addr: IpAddr::V4(Ipv4Addr::new(172, 16, 13, 200)),
    slot: 0,
  };

  let hint = ConsumerHint {
    tag: String::from("test"),
    data_size: 6,
    otrpi: 1_100_000,
    rpi: 1_000_000
  };

  service.add_consumer(addr, hint, handler)
    .unwrap();

  // Wait for enter
  let mut stdout = stdout();
  stdout.write(b"Press ENTER to close...").unwrap();
  stdout.flush().unwrap();
  stdin().read(&mut [0]).unwrap();
}