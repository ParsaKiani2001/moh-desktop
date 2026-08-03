use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process;
#[derive(serde::Deserialize)]
struct HubMessage {
    topic: String,
    payload: serde_json::Value,
}
pub struct HubController {
    stream: UnixStream,
}

impl HubController {
    fn socket_path() -> String {
        let runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
        format!("{runtime}/moh-event-hub.sock")
    }

    fn hub_connect()    -> Result<UnixStream, Box<dyn std::error::Error>>{
        let stream = UnixStream::connect(Self::socket_path())?;
        
        println!("[wallpaper] connected");
        Ok(stream)
    }
    pub fn new() -> Result<Self,Box<dyn std::error::Error>> {
        let mut  stram = Self::hub_connect()?;
        writeln!(stram,r#"{{"kind":"Register","topics":["system.exit"]}}"#)?;

        Ok(Self{
            stream:stram
        })
    }
    pub fn checker(&self)-> Result<(),Box<dyn std::error::Error>> {
        let stream = self.stream.try_clone()?;
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stream);

            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap() == 0 {
                    break;
                }
                let msg: HubMessage = serde_json::from_str(&line).unwrap();
                if msg.topic == "system.exit" {
                    println!("wallpaper Exit");
                    std::process::exit(0);
                }
            }
        });
        Ok(())
    }
}

