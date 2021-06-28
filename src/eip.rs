use byteorder::{LittleEndian, WriteBytesExt};
use rand::Rng;







/* Register the PLC */
pub fn build_register_session() -> Vec<u8> {
  const EIPCommand: u16 = 0x0065;
  const EIPLength: u16 = 0x0004;
  const EIPSessionHandle: u32 = 0x00;
  const EIPStatus: u32 = 0x0000;
  const EIPContext: u64 = 0x00;
  const EIPOptions: u32 = 0x0000;

  const EIPProtocolVersion: u16 = 0x01;
  const EIPOptionFlag: u16 = 0x00;

  let mut registerSession = Vec<u8>::with_capacity(24);

  registerSession.write_u16::<LittleEndian>(EIPCommand);
  registerSession.write_u16::<LittleEndian>(EIPLength);
  registerSession.write_u32::<LittleEndian>(EIPSessionHandle);
  registerSession.write_u32::<LittleEndian>(EIPStatus);
  registerSession.write_u64::<LittleEndian>(EIPContext);
  registerSession.write_u32::<LittleEndian>(EIPOptions);
  registerSession.write_u16::<LittleEndian>(EIPProtocolVersion);
  registerSession.write_u16::<LittleEndian>(EIPOptionFlag);

  return registerSession;
}








/* Create Forward Open */
pub fn build_forward_open_packet(session_handle: u32, hint: ConsumerHint) -> Vec<u8>{
  // Get bytes
  let forward_open = build_cip_forward_open(hint: ConsumerHint);
  let header = build_eip_send_rr_data_header(forward_open.len(), session_handle);

  // Concatenate
  header.append(forward_open);

  return header;
}


fn build_eip_send_rr_data_header(frame_len: u16, session_handle: u32) -> Vec<u8> {
  const EIPCommand: u16 = 0x6F;
  const EIPLength: u16 = 16+frame_len;
  const EIPSessionHandle: u32 = session_handle;
  const EIPStatus: u32 = 0x00;
  const EIPContext: u64 = 0x8000004a00000000;
  const EIPOptions: u32 = 0x00;

  const EIPInterfaceHandle: u32 = 0x00;
  const EIPTimeout: u16 = 0x00;
  const EIPItemCount: u16 = 0x02;
  const EIPItem1Type: u16 = 0x00;
  const EIPItem1Length: u16 = 0x00;
  const EIPItem2Type: u16 = 0xB2;
  const EIPItem2Length: u16 = frameLen;

  let mut header = Vec<u8>::with_capacity(40);

  header.write_u16::<LittleEndian>(EIPCommand);
  header.write_u16::<LittleEndian>(EIPLength);
  header.write_u32::<LittleEndian>(EIPSessionHandle);
  header.write_u32::<LittleEndian>(EIPStatus);
  header.write_u64::<LittleEndian>(EIPContext);
  header.write_u32::<LittleEndian>(EIPOptions);
  header.write_u32::<LittleEndian>(EIPInterfaceHandle);
  header.write_u16::<LittleEndian>(EIPTimeout);
  header.write_u16::<LittleEndian>(EIPItemCount);
  header.write_u16::<LittleEndian>(EIPItem1Type);
  header.write_u16::<LittleEndian>(EIPItem1Length);
  header.write_u16::<LittleEndian>(EIPItem2Type);
  header.write_u16::<LittleEndian>(EIPItem2Length);

  return header;
}

fn build_cip_forward_open(hint: ConsumerHint) {
  const CIPService: u8 = 0x54;
  const CIPPathSize: u8 = 0x02;
  const CIPClassType: u8 = 0x20;
  const CIPClass: u8 = 0x06;
  const CIPInstanceType: u8 = 0x24;
  const CIPInstance: u8 = 0x01;
  const CIPPriority: u8 = 0x0A;
  const CIPTimeoutTicks: u8 = 0x0e;

  // Random number generator
  let mut rng = rand::thread_rng();

  const CIPOTConnectionID: u32 = 0x00;
  const CIPTOConnectionID: u32 = rng.gen_range(0..65000);
  const CIPConnectionSerialNumber: u16 = rng.gen_range(0..65000);
  const CIPVendorID: u16 = 0x01;
  const CIPOriginatorSerialNumber: u32 = 42;
  const CIPMultiplier: u32 = 0x00;
  const CIPOTRPI: u32 = hint.otrpi * 1000;
  const CIPOTNetworkConnectionParameters = 0x4802;
  const CIPTORPI: u32 = hint.rpi * 1000;
  const CIPTONetworkConnectionParameters: u16 = 0x4800 + hint.datasize;
  
  const CIPTransportTrigger: u8 = 0x81;

  // Build bytes
  let mut forward_open = Vec<u8>::with_capacity(328);

  forward_open.write_u8::<LittleEndian>(CIPService);
  forward_open.write_u8::<LittleEndian>(CIPPathSize);
  forward_open.write_u8::<LittleEndian>(CIPClassType);
  forward_open.write_u8::<LittleEndian>(CIPClass);
  forward_open.write_u8::<LittleEndian>(CIPInstanceType);
  forward_open.write_u8::<LittleEndian>(CIPInstance);
  forward_open.write_u8::<LittleEndian>(CIPPriority);
  forward_open.write_u8::<LittleEndian>(CIPTimeoutTicks)
  forward_open.write_u32::<LittleEndian>(CIPOTConnectionID);
  forward_open.write_u32::<LittleEndian>(CIPTOConnectionID);
  forward_open.write_u16::<LittleEndian>(CIPConnectionSerialNumber);
  forward_open.write_u16::<LittleEndian>(CIPVendorID);
  forward_open.write_u32::<LittleEndian>(CIPOriginatorSerialNumber);
  forward_open.write_u32::<LittleEndian>(CIPMultiplier);
  forward_open.write_u32::<LittleEndian>(CIPOTRPI);
  forward_open.write_u16::<LittleEndian>(CIPOTNetworkConnectionParameters);
  forward_open.write_u32::<LittleEndian>(CIPTORPI);
  forward_open.write_u16::<LittleEndian>(CIPTONetworkConnectionParameters);
  forward_open.write_u8::<LittleEndian>(CIPTransportTrigger);



  # add the connection path
  path_size, path = self._connection_path()
  connection_path = pack('<B', int(path_size/2))

  connection_path += path
  return ForwardOpen + connection_path

}

fn build_connection_path(hint: ConsumerHint) -> Vec<u8> {
  const PortSegment = 0x01 #b;
  const LinkAddress = 0x00 #b;
  const KeySegment = 0x34 #b;
  const KeyFormat = 0x04 #b;
  const VendorID = 0x00 #h;
  const DeviceType = 0x00 #h;
  const ProductCode = 0x00 #h;
  const MajorRevision = 0x00 #b;
  const MinorRevision = 0x00 #b;

  // Build bytes
  let mut path = Vec<u8>::with_capacity(96);

  path.write_u8::<LittleEndian>(PortSegment);
  path.write_u8::<LittleEndian>(LinkAddress);
  path.write_u8::<LittleEndian>(KeySegment);
  path.write_u8::<LittleEndian>(KeyFormat);
  path.write_u16::<LittleEndian>(VendorID);
  path.write_u16::<LittleEndian>(DeviceType);
  path.write_u16::<LittleEndian>(ProductCode);
  path.write_u8::<LittleEndian>(MajorRevision);
  path.write_u8::<LittleEndian>(MinorRevision);

  // Add tag
  path.push( build_tag_ioi(hint.tag, 160) )

  return path;
}

fn build_tag_ioi(tag: String, data_type) -> Vec<u8> {
  let mut ioi = vec![];
  for tag_component in tag.split(".") {
    
  }
}