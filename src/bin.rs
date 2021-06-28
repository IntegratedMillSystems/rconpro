use rconpro::{eip, ConsumerHint};

fn main() {
  let hint = ConsumerHint {
    tag: String::from("Test"),
    rpi: 1000,
    otrpi: 1100,
    data_size: 6
  };

  println!("{:?}", eip::build_forward_open_packet(1234, hint));
  
}
