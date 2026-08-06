use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::messages::{IncomingMessage, Message};

struct Subscriber {
    topics: Vec<String>,
    writer: UnixStream,
}

pub struct HubServer {
    listener: UnixListener,
    subscribers: Arc<Mutex<HashMap<u64, Subscriber>>>,
    next_id: u64,
}

impl HubServer {
    pub fn new(socket_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let _ = std::fs::remove_file(socket_path);
        let listener = UnixListener::bind(socket_path)?;
        
        Ok(Self {
            listener,
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            next_id: 0,
        })
    }
    
    pub fn run<F>(&mut self, on_exit: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn() + Send + Clone + 'static
    {
        println!("[hub] Server listening");
        
        for stream in self.listener.incoming().flatten() {
            let id = self.next_id;
            self.next_id += 1;
            
            let subscribers = self.subscribers.clone();
            let writer = stream.try_clone()?;
            
            subscribers.lock().unwrap().insert(id, Subscriber {
                topics: vec![],
                writer,
            });
            
            let on_exit_clone = on_exit.clone();
            
            thread::spawn(move || {
                let reader = BufReader::new(stream);
                
                for line in reader.lines().flatten() {
                    if line.trim().is_empty() { continue; }
                    
                    match serde_json::from_str::<Message>(&line) {
                        Ok(Message::Register { topics }) => {
                            if let Some(sub) = subscribers.lock().unwrap().get_mut(&id) {
                                sub.topics = topics;
                                println!("[hub] Client {} registered for {:?}", id, sub.topics);
                            }
                        }
                        Ok(Message::Publish { topic, payload }) => {
                            println!("[hub] Publish: topic={}", topic);
                            
                            let msg = IncomingMessage { topic: topic.clone(), payload: payload.clone() };
                            let out = serde_json::to_string(&msg).unwrap();
                            
                            let mut subs = subscribers.lock().unwrap();
                            for (_, sub) in subs.iter_mut() {
                                if sub.topics.contains(&topic) {
                                    let _ = writeln!(sub.writer, "{}", out);
                                }
                            }
                            
                            if topic == "system.exit" {
                                thread::sleep(std::time::Duration::from_millis(300));
                                on_exit_clone();
                                return;
                            }
                        }
                        Err(e) => eprintln!("[hub] Bad message: {} -- {}", e, line),
                    }
                }
                
                subscribers.lock().unwrap().remove(&id);
                println!("[hub] Client {} disconnected", id);
            });
        }
        
        Ok(())
    }
}

fn on_exit() {} // dummy for Send trait