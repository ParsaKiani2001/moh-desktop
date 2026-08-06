use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::messages::{IncomingMessage, Message};

type MessageHandler = Box<dyn Fn(IncomingMessage) + Send + Sync>;

pub struct HubClient {
    stream: UnixStream,
    handlers: Arc<Mutex<Vec<MessageHandler>>>,
    app_name: String
}


impl HubClient {
    pub fn connect(app_name: &str, socket_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let max_retries = 10;
        let retry_delay = Duration::from_millis(500);
        
        for attempt in 1..=max_retries {
            match UnixStream::connect(socket_path) {
                Ok(stream) => {
                    println!("[{}] Connected to event-hub", app_name);
                    return Ok(Self {
                        stream,
                        handlers: Arc::new(Mutex::new(Vec::new())),
                        app_name: app_name.to_string(),
                    });
                }
                Err(e) => {
                    if attempt == max_retries {
                        return Err(format!(
                            "[{}] Failed to connect to event-hub after {} attempts: {}",
                            app_name, max_retries, e
                        ).into());
                    }
                    println!(
                        "[{}] Waiting for event-hub... (attempt {}/{})",
                        app_name, attempt, max_retries
                    );
                    thread::sleep(retry_delay);
                }
            }
        }
        
        unreachable!()
    }
    
    /// اتصال ساده بدون retry
    pub fn connect_simple(app_name: &str, socket_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let stream = UnixStream::connect(socket_path).map_err(|e| {
            format!("[{}] Connection to event-hub failed: {}", app_name, e)
        })?;
        
        println!("[{}] Connected to event-hub", app_name);
        
        Ok(Self {
            stream,
            handlers: Arc::new(Mutex::new(Vec::new())),
            app_name: app_name.to_string(),
        })
    }
    
    pub fn register(&mut self, topics: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let msg = Message::register(topics);
        writeln!(self.stream, "{}\n", msg.to_json()).map_err(|e| {
            format!("[{}] Failed to register: {}", self.app_name, e)
        })?;
        Ok(())
    }
    
    pub fn publish(&mut self, topic: &str, payload: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        let msg = Message::publish(topic, payload);
        writeln!(self.stream, "{}\n", msg.to_json()).map_err(|e| {
            format!("[{}] Failed to publish: {}", self.app_name, e)
        })?;
        Ok(())
    }
    
    pub fn on_message<F>(&self, handler: F) 
    where 
        F: Fn(IncomingMessage) + Send + Sync + 'static 
    {
        self.handlers.lock().unwrap().push(Box::new(handler));
    }
    
    pub fn start_listener(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stream = self.stream.try_clone()?;
        let handlers = self.handlers.clone();
        let app_name = self.app_name.clone();
        
        thread::spawn(move || {
            let reader = BufReader::new(stream);
            for line in reader.lines().flatten() {
                if line.trim().is_empty() { continue; }
                
                if let Ok(msg) = serde_json::from_str::<IncomingMessage>(&line) {
                    println!("[{}] Received: topic={}", app_name, msg.topic);
                    let handlers = handlers.lock().unwrap();
                    for handler in handlers.iter() {
                        handler(msg.clone());
                    }
                }
            }
        });
        
        Ok(())
    }
}

impl Drop for HubClient {
    fn drop(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}