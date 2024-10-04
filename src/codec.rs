use bytes::BytesMut;
use tokio::io;
use tokio_io::_tokio_codec::Decoder;
use tokio_io::codec::Encoder;
use crate::client_command::Command;
use crate::error::FtpError;
use crate::ftp_response::Response;
use crate::utils::find_crlf;

pub struct FtpCodec;
pub struct BytesCodec;


impl Decoder for FtpCodec {
    type Item = Command;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Command>> {
        if let Some(index) = find_crlf(buf) {
            let line = buf.split_to(index);
            // To remove \r\n
            buf.split_to(2);


            Command::new(line.to_vec())
                .map(|command| Some(command))
                .map_err(FtpError::to_io_error)
        } else {
            Ok(None)
        }
    }
}

impl Encoder for FtpCodec {
    type Item = Response;
    type Error = io::Error;

    fn encode(&mut self, resp: Response, buf: &mut BytesMut) -> io::Result<()> {
        let mut buffer = vec![];

        if resp.message.is_empty() {
            write!(buffer, "{}\r\n",resp.code as u32)?;
        } else {
            write!(buffer, "{} {}\r\n", resp.code as u32, resp.message)?;
        }
        buf.extend(&buffer);
        Ok(())
    }
}