use bytes::BytesMut;
use tokio::io;
use tokio_io::_tokio_codec::Decoder;
use crate::client_command::Command;
use crate::utils::find_crlf;

pub struct FtpCodec;
pub struct BytesCodec;


impl Decoder for FtpCodec {
    type Item = Command;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(index) = find_crlf(buf) {
            let line = buf.split_to(index);
            buf.split_to(2); // remove the CRLF
            // let line = String::from_utf8(line.to_vec()).unwrap();
            let command = Command::new(line);
            Ok(Some(command))
        } else {
            Ok(None)
        }
    }
}