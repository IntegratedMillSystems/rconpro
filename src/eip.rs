use std::convert::TryInto;
use byteorder::{LittleEndian, WriteBytesExt};
use rand::Rng;
use encoding::{Encoding, EncoderTrap};
use encoding::all::UTF_8;

use crate::ConsumerHint;




/* Register the PLC */
pub fn build_register_session() -> Vec<u8> {
  const EIP_COMMAND: u16 = 0x0065;
  const EIP_LENGTH: u16 = 0x0004;
  const EIP_SESSION_HANDLE: u32 = 0x00;
  const EIP_STATUS: u32 = 0x0000;
  const EIP_CONTEXT: u64 = 0x00;
  const EIP_OPTIONS: u32 = 0x0000;

  const EIP_PROTOCOL_VERSION: u16 = 0x01;
  const EIP_OPTION_FLAG: u16 = 0x00;

  let mut register_session = Vec::<u8>::with_capacity(24);

  register_session.write_u16::<LittleEndian>(EIP_COMMAND).unwrap();
  register_session.write_u16::<LittleEndian>(EIP_LENGTH).unwrap();
  register_session.write_u32::<LittleEndian>(EIP_SESSION_HANDLE).unwrap();
  register_session.write_u32::<LittleEndian>(EIP_STATUS).unwrap();
  register_session.write_u64::<LittleEndian>(EIP_CONTEXT).unwrap();
  register_session.write_u32::<LittleEndian>(EIP_OPTIONS).unwrap();
  register_session.write_u16::<LittleEndian>(EIP_PROTOCOL_VERSION).unwrap();
  register_session.write_u16::<LittleEndian>(EIP_OPTION_FLAG).unwrap();

  return register_session;
}

#[test]
fn test_build_register_session() {
  assert_eq!(
    build_register_session(),
    vec![101, 0, 4, 0, 0, 0, 0,
         0, 0, 0, 0, 0, 0, 0, 0,
         0, 0, 0, 0, 0, 0, 0, 0,
         0, 1, 0, 0, 0]
  );
}








/* Create Forward Open */
pub fn build_forward_open_packet(session_handle: u32, hint: &ConsumerHint) -> Vec<u8>{
  // Get bytes
  let mut forward_open = build_cip_forward_open(hint);
  let mut header = build_eip_send_rr_data_header(
    forward_open.len().try_into().unwrap(),
    session_handle
  );

  // Concatenate
  header.append(&mut forward_open);

  return header;
}


fn build_eip_send_rr_data_header(frame_len: u16, session_handle: u32) -> Vec<u8> {
  const EIP_COMMAND: u16 = 0x6F;
  let eip_length: u16 = 16+frame_len;
  const EIP_STATUS: u32 = 0x00;
  const EIP_CONTEXT: u64 = 0x8000004a00000000;
  const EIP_OPTIONS: u32 = 0x00;

  const EIP_INTERFACE_HANDLE: u32 = 0x00;
  const EIP_TIMEOUT: u16 = 0x00;
  const EIP_ITEM_COUNT: u16 = 0x02;
  const EIP_ITEM_1_TYPE: u16 = 0x00;
  const EIP_ITEM_1_LENGTH: u16 = 0x00;
  const EIP_ITEM_2_TYPE: u16 = 0xB2;
  let eip_item_2_length: u16 = frame_len;

  let mut header = Vec::<u8>::with_capacity(40);

  header.write_u16::<LittleEndian>(EIP_COMMAND).unwrap();
  header.write_u16::<LittleEndian>(eip_length).unwrap();
  header.write_u32::<LittleEndian>(session_handle).unwrap();
  header.write_u32::<LittleEndian>(EIP_STATUS).unwrap();
  header.write_u64::<LittleEndian>(EIP_CONTEXT).unwrap();
  header.write_u32::<LittleEndian>(EIP_OPTIONS).unwrap();
  header.write_u32::<LittleEndian>(EIP_INTERFACE_HANDLE).unwrap();
  header.write_u16::<LittleEndian>(EIP_TIMEOUT).unwrap();
  header.write_u16::<LittleEndian>(EIP_ITEM_COUNT).unwrap();
  header.write_u16::<LittleEndian>(EIP_ITEM_1_TYPE).unwrap();
  header.write_u16::<LittleEndian>(EIP_ITEM_1_LENGTH).unwrap();
  header.write_u16::<LittleEndian>(EIP_ITEM_2_TYPE).unwrap();
  header.write_u16::<LittleEndian>(eip_item_2_length).unwrap();

  return header;
}

#[test]
fn test_build_eip_send_rr_data_header() {
  assert_eq!(
    build_eip_send_rr_data_header(0, 0),
    vec![111, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 74, 0, 0, 128, 0, 0,
         0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 178, 0, 0, 0]
  )
}


fn build_cip_forward_open(hint: &ConsumerHint) -> Vec<u8> {
  const CIP_SERVICE: u8 = 0x54;
  const CIP_PATH_SIZE: u8 = 0x02;
  const CIP_CLASS_TYPE: u8 = 0x20;
  const CIP_CLASS: u8 = 0x06;
  const CIP_INSTANCE_TYPE: u8 = 0x24;
  const CIP_INSTANCE: u8 = 0x01;
  const CIP_PRIORITY: u8 = 0x0A;
  const CIP_TIMEOUT_TICKS: u8 = 0x0e;

  // Random number generator
  let mut rng = rand::thread_rng();

  const CIP_OT_CONNECTION_ID: u32 = 0x00;
  let cip_to_connection_id: u32 = rng.gen_range(0..65000);
  let cip_connection_serial_number: u16 = rng.gen_range(0..65000);
  const CIP_VENDOR_ID: u16 = 0x01;
  const CIP_ORIGINATOR_SERIAL_NUMBER: u32 = 42;
  const CIP_MULTIPLIER: u32 = 0x00;
  let cip_ot_rpi: u32 = hint.otrpi.try_into().unwrap();
  const CIP_OT_NETWORK_CONNECTION_PARAMETERS: u16 = 0x4802;
  let cip_to_rpi: u32 = hint.rpi.try_into().unwrap();
  let cip_to_network_connection_parameters: u16 = (0x4800 + hint.data_size).try_into().unwrap();
  
  const CIP_TRANSPORT_TRIGGER: u8 = 0x81;

  // Build bytes
  let mut forward_open = Vec::<u8>::with_capacity(328);

  forward_open.write_u8(CIP_SERVICE).unwrap();
  forward_open.write_u8(CIP_PATH_SIZE).unwrap();
  forward_open.write_u8(CIP_CLASS_TYPE).unwrap();
  forward_open.write_u8(CIP_CLASS).unwrap();
  forward_open.write_u8(CIP_INSTANCE_TYPE).unwrap();
  forward_open.write_u8(CIP_INSTANCE).unwrap();
  forward_open.write_u8(CIP_PRIORITY).unwrap();
  forward_open.write_u8(CIP_TIMEOUT_TICKS).unwrap();
  forward_open.write_u32::<LittleEndian>(CIP_OT_CONNECTION_ID).unwrap();
  forward_open.write_u32::<LittleEndian>(cip_to_connection_id).unwrap();
  forward_open.write_u16::<LittleEndian>(cip_connection_serial_number).unwrap();
  forward_open.write_u16::<LittleEndian>(CIP_VENDOR_ID).unwrap();
  forward_open.write_u32::<LittleEndian>(CIP_ORIGINATOR_SERIAL_NUMBER).unwrap();
  forward_open.write_u32::<LittleEndian>(CIP_MULTIPLIER).unwrap();
  forward_open.write_u32::<LittleEndian>(cip_ot_rpi).unwrap();
  forward_open.write_u16::<LittleEndian>(CIP_OT_NETWORK_CONNECTION_PARAMETERS).unwrap();
  forward_open.write_u32::<LittleEndian>(cip_to_rpi).unwrap();
  forward_open.write_u16::<LittleEndian>(cip_to_network_connection_parameters).unwrap();
  forward_open.write_u8(CIP_TRANSPORT_TRIGGER).unwrap();

  // Add the connection path
  let mut path = build_connection_path(hint);
  forward_open.write_u8( (path.len()/2).try_into().unwrap() ).unwrap();
  forward_open.append(&mut path);

  return forward_open;
}


fn build_connection_path(hint: &ConsumerHint) -> Vec<u8> {
  const PORT_SEGMENT: u8 = 0x01;
  const LINK_ADDRESS: u8 = 0x00;
  const KEY_SEGMENT: u8 = 0x34;
  const KEY_FORMAT: u8 = 0x04;
  const VENDOR_ID: u16 = 0x00;
  const DEVICE_TYPE: u16 = 0x00;
  const PRODUCT_CODE: u16 = 0x00;
  const MAJOR_REVISION: u8 = 0x00;
  const MINOR_REVISION: u8 = 0x00;

  // Build bytes
  let mut path = Vec::<u8>::with_capacity(96);

  path.write_u8(PORT_SEGMENT).unwrap();
  path.write_u8(LINK_ADDRESS).unwrap();
  path.write_u8(KEY_SEGMENT).unwrap();
  path.write_u8(KEY_FORMAT).unwrap();
  path.write_u16::<LittleEndian>(VENDOR_ID).unwrap();
  path.write_u16::<LittleEndian>(DEVICE_TYPE).unwrap();
  path.write_u16::<LittleEndian>(PRODUCT_CODE).unwrap();
  path.write_u8(MAJOR_REVISION).unwrap();
  path.write_u8(MINOR_REVISION).unwrap();

  // Add tag
  path.append( &mut build_tag_ioi(&hint.tag) );

  return path;
}

#[test]
fn test_build_connection_path() {
  let hint = ConsumerHint {
    tag: String::from("Test"),
    data_size: 6,
    rpi: 1000,
    otrpi: 1100
  };

  assert_eq!(
    build_connection_path(&hint),
    vec![1, 0, 52, 4, 0, 0, 0, 0, 0, 0, 0, 0, 145, 4, 84, 101, 115, 116]
  );
}


fn build_tag_ioi(tag: &str) -> Vec<u8> {
  /*
  Fron pyconpro:

  The tag IOI is basically the tag name assembled into
  an array of bytes structured in a way that the PLC will
  understand.  It's a little crazy, but we have to consider the
  many variations that a tag can be:
  TagName (DINT)
  TagName.1 (Bit of DINT) <- not handling this!
  TagName.Thing (UDT)
  TagName[4].Thing[2].Length (more complex UDT) <- we're not going to handle this for now
  We also might be reading arrays, a bool from arrays (atomic), strings.
      Oh and multi-dim arrays, program scope tags...
  */

  let mut ioi = vec![];
  for tag_component in tag.split(".") {
    ioi.write_u8(0x91).unwrap();
    ioi.write_u8(tag_component.len().try_into().unwrap() ).unwrap();
    ioi.append( &mut UTF_8.encode(tag_component, EncoderTrap::Strict).unwrap() );

    if tag_component.len() % 2 == 1 {
      ioi.push(0x00);
    }
  }

  return ioi;
}

#[test]
fn test_build_tag_ioi() {
  assert_eq!(
    build_tag_ioi("Test"),
    vec![145, 4, 84, 101, 115, 116]
  );
}







/* Keep-Alive packet */
pub fn build_response_packet(ot_connection_id: u32, sequence_count: u32) -> Vec<u8> {
  const ITEM_COUNT: u16 = 0x02;
  const TYPE_ID: u16 = 0x8002;
  const LENGTH: u16 = 0x08;
  const CONN_DATA: u16 = 0x00b1;
  const DATA_LENGTH: u16 = 0x02;
  const SEQUENCE_COUNT: u16 = 1;

  let mut payload = Vec::<u8>::with_capacity(20);

  payload.write_u16::<LittleEndian>(ITEM_COUNT).unwrap();
  payload.write_u16::<LittleEndian>(TYPE_ID).unwrap();
  payload.write_u16::<LittleEndian>(LENGTH).unwrap();

  payload.write_u32::<LittleEndian>(ot_connection_id).unwrap();
  payload.write_u32::<LittleEndian>(sequence_count).unwrap();

  payload.write_u16::<LittleEndian>(CONN_DATA).unwrap();
  payload.write_u16::<LittleEndian>(DATA_LENGTH).unwrap();
  payload.write_u16::<LittleEndian>(SEQUENCE_COUNT).unwrap();

  return payload;
}

#[test]
fn test_build_response_packet() {
  assert_eq!(
    build_response_packet(0, 0),
    vec![2, 0, 2, 128, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 177, 0, 2, 0, 1, 0]
  );
}







/* Get Tag List */
pub fn build_get_tag_list_query() {
  
}