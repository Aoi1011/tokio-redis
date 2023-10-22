mod get;
pub use get::Get;

mod unknown;
pub use unknown::Unknown;

use crate::{Frame, parse::Parse};

#[derive(Debug)]
pub enum Command {
    Get(Get),
    Unknown(Unknown),
}

impl Command {
    pub fn from_frame(frame: Frame) -> crate::Result<()> {
        let mut parse = Parse::new(frame)?;

        let command_name = parse.next_string()?.to_lowercase();

        // let command = match command_name[..] {
        //     "get" => Command::Get(Get::parse_frames(&mut parse)?),
        //     _ => {
        //     }
        // }
        Ok(())
    }
}
