use std::sync::mpsc;

use common::{socket_path, HubClient};
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

#[derive(Debug, serde::Deserialize)]
struct UIMessage {
    action: String,
    #[serde(default)]
    dx: i32,
    #[serde(default)]
    dy: i32,
}

#[derive(Debug, Clone)]
enum Command {
    WindowOpened { id: u32, title: String },
    WindowClosed { id: u32 },
    WindowFocused { id: u32 },
    ThemeChanged { theme: String },
    Exit,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[ui-service] Starting...");

    let mut hub = HubClient::connect("ui-service", &socket_path())?;
    hub.register(vec![
        "system.exit",
        "window.opened",
        "window.closed",
        "window.focused",
        "theme.change",
    ])?;

    let (tx, rx) = mpsc::channel::<Command>();

    hub.on_message(move |msg| {
        println!("[ui-service] got: topic={}", msg.topic);
        let cmd = match msg.topic.as_str() {
            "system.exit" => Command::Exit,
            "window.opened" => {
                let id = msg.payload.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let title = msg.payload.get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Window")
                    .to_string();
                Command::WindowOpened { id, title }
            }
            "window.closed" => {
                let id = msg.payload.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                Command::WindowClosed { id }
            }
            "window.focused" => {
                let id = msg.payload.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                Command::WindowFocused { id }
            }
            "theme.change" => {
                let theme = msg.payload.get("theme")
                    .and_then(|v| v.as_str())
                    .unwrap_or("dark")
                    .to_string();
                Command::ThemeChanged { theme }
            }
            _ => return,
        };
        let _ = tx.send(cmd);
    });
    hub.start_listener()?;

    let event_loop = EventLoopBuilder::new().build();

    let window = WindowBuilder::new()
        .with_title("MOH Desktop - UI Service")
        .with_inner_size(tao::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)?;

    let (ipc_tx, ipc_rx) = mpsc::channel::<String>();

    // ✅ مسیر داینامیک به index.html
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let index_path = format!("file://{}/web/index.html", manifest_dir);
    println!("[ui-service] Loading: {}", index_path);

    // ✅ API درست wry 0.35:
    // new() → WebViewBuilder
    // with_url() → Result<WebViewBuilder> (نیاز به ?)
    // with_ipc_handler() → WebViewBuilder
    // build() → Result<WebView> (نیاز به ?)
    let webview = WebViewBuilder::new()
    .with_url(&index_path)
    .with_ipc_handler(move |req| {
        println!("[ui-service] IPC: {:?}", req);
        let msg = req.body().clone();
        let _ = ipc_tx.send(msg);
    })
    .build(&window)?;

    println!("[ui-service] WebView created");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::NewEvents(StartCause::Init) => {
                println!("[ui-service] Initialized");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                while let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        Command::Exit => *control_flow = ControlFlow::Exit,
                        Command::WindowOpened { title, .. } => {
                            let js = format!(
                                "handleWMMessage({{type: 'set_title', title: '{}'}})",
                                title.replace("'", "\\'")
                            );
                            let _ = webview.evaluate_script(&js);
                        }
                        Command::WindowFocused { .. } => {
                            let js = "handleWMMessage({type: 'set_focus', focused: true})";
                            let _ = webview.evaluate_script(js);
                        }
                        Command::ThemeChanged { theme } => {
                            let js = format!(
                                "handleWMMessage({{type: 'set_theme', theme: '{}'}})",
                                theme
                            );
                            let _ = webview.evaluate_script(&js);
                        }
                        _ => {}
                    }
                }

                while let Ok(msg) = ipc_rx.try_recv() {
                    if let Ok(ui_msg) = serde_json::from_str::<UIMessage>(&msg) {
                        let topic = match ui_msg.action.as_str() {
                            "close" => "window.close_request",
                            "minimize" => "window.minimize_request",
                            "maximize" => "window.maximize_request",
                            "move" => "window.move_request",
                            "resize" => "window.resize_request",
                            "drag_end" => "window.drag_end",
                            _ => continue,
                        };
                        println!("[ui-service] Action: {}", topic);
                    }
                }
            }
            _ => {}
        }
    });
}