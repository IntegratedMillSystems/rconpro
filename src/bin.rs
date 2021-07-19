use rconpro::{Service, EipAddr, ConsumerHint, ConsumerQueue};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

fn main() {
  let mut service = Service::new();
  service.start().unwrap();

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

  let data = Arc::new(ConsumerQueue::new());

  service.add_consumer(addr, hint, &data)
    .unwrap();

  loop {
    while !data.is_empty() {
      println!("{:?}", data.pop());
    }
  }
}