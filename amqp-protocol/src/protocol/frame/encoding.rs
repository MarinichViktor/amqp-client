// use std::io::Write;
// use crate::protocol::encoder::Encode;
// use crate::protocol::method::{Method, ServerProperty};
// use crate::response;


// pub fn encode_method(method: Method) -> response::Result<Vec<u8>> {
//     let mut frame = vec![];
//     // Method frame type
//     frame.write_byte(1)?;
//     let frame_type = 1;
//
//     // frame.write_byte(0xCE as u8)?;
//     match method {
//         Method::StartOkMethod(m) => {
//             let mut buff = vec![];
//             // class id
//             buff.write_short(10)?;
//             // method id
//             buff.write_short(11)?;
//
//             buff.write_prop_table(m.client_properties)?;
//             buff.write_short_str(m.mechanism)?;
//             buff.write_long_str(m.response)?;
//             buff.write_short_str(m.locale)?;
//
//             frame = encode_frame(frame_type, 0, buff)?;
//         },
//         _ => {
//             panic!("Unsupported method type")
//         }
//     };
//
//     Ok(frame)
// }

// pub fn encode_frame(frame_type: u8, chan: u16, body: Vec<u8>) -> response::Result<Vec<u8>> {
//     let mut frame = vec![];
//     // Frame type
//     frame.write_byte(frame_type)?;
//     // Frame chan
//     frame.write_ushort(chan)?;
//     // Body size and body
//     frame.write_uint(body.len() as u32)?;
//     frame.write(& body)?;
//
//     // EOF char
//     frame.write_byte(206 as u8)?;
//     // frame.write_byte(0xCE as u8)?;
//
//     Ok(frame)
// }

// pub fn encode_method_frame(chan: u16, args: Vec<ServerProperty>) -> response::Result<Vec<u8>> {
//     let mut raw_frame = vec![];
//
//     // Frame type
//     raw_frame.write_byte(1)?;
//     // Frame chan
//     raw_frame.write_ushort(chan)?;
//     // Body size and body
//     let raw_args = encode_method_args(args)?;
//     raw_frame.write_uint(raw_args.len() as u32)?;
//     raw_frame.write(&raw_args)?;
//
//     // EOF char
//     raw_frame.write_byte(206 as u8)?;
//     // frame.write_byte(0xCE as u8)?;
//
//     Ok(raw_frame)
// }

// pub fn encode_method_args(args: Vec<ServerProperty>) -> response::Result<Vec<u8>> {
//     let mut raw_args = vec![];
//
//     for arg in args {
//         raw_args.write_argument(arg)?;
//     }
//
//     Ok(raw_args)
// }
