pub mod tokens;

use std::sync::{Arc, RwLock};

use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};
use tokens::Tokens;
use wry::{WebViewBuilder, WebViewBuilderExtWindows};

pub const RIOT_AUTH_PAGE: &str = concat!(
    "https://auth.riotgames.com/authorize?",
    "redirect_uri=https%3A%2F%2Fplayvalorant.com%2Fopt_in&",
    "client_id=play-valorant-web-prod&",
    "response_type=token%20id_token&",
    "scope=account%20openid&",
    "nonce=1"
);

pub fn login_popup(login_page: &str) -> Option<Tokens> {
    let mut event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Riot Login")
        .with_inner_size(LogicalSize::new(800, 950))
        .with_resizable(false)
        .build(&event_loop)
        .expect("window should create");

    let shared_data = Arc::new(RwLock::new(String::new()));
    let c_tokens_shared = shared_data.clone();

    let event_loop_proxy = event_loop.create_proxy();
    let _webview = WebViewBuilder::new(&window)
        .with_url(login_page)
        .with_https_scheme(true)
        .with_navigation_handler(move |url| {
            if url.contains("access_token") {
                let mut set_tokens = c_tokens_shared.write().unwrap();
                *set_tokens = url.to_string();

                event_loop_proxy
                    .send_event(())
                    .expect("event should be sent");

                return false;
            }

            true
        })
        .build()
        .expect("webview should build");

    let exit_code = event_loop.run_return(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::ExitWithCode(1),
        Event::UserEvent(_) => *control_flow = ControlFlow::Exit,
        _ => *control_flow = ControlFlow::Wait,
    });

    println!("Window exited with exit code {exit_code}");

    let get_token = shared_data.read().unwrap();

    url::Url::parse(&*get_token)
        .ok()
        .and_then(|u| u.fragment().map(|str| str.to_string()))
        .map(|str| serde_urlencoded::from_str(&str).expect("fragment should contain tokens"))
}
