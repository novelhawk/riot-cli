pub mod tokens;

use std::{
    cell::OnceCell,
    mem,
    path::Path,
    ptr,
    rc::Rc,
    sync::{Arc, RwLock},
};

use cookie::{time::OffsetDateTime, Cookie, Expiration};
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    rwh_06::{HasWindowHandle, RawWindowHandle},
    window::WindowBuilder,
};
use tokens::Tokens;
use webview2::{check_hresult, Controller, Environment};
use webview2_sys::{ICoreWebView2CookieManagerVTable, ICoreWebView2_2};
use winapi::{
    shared::{windef::HWND, winerror::E_FAIL},
    um::winuser::GetClientRect,
};
use wry::{WebViewBuilder, WebViewBuilderExtWindows};

pub const RIOT_AUTH_PAGE: &str = concat!(
    "https://auth.riotgames.com/authorize?",
    "redirect_uri=https%3A%2F%2Fplayvalorant.com%2Fopt_in&",
    "client_id=play-valorant-web-prod&",
    "response_type=token%20id_token&",
    "scope=account%20openid&",
    "nonce=1"
);

pub fn login_popup(profile_folder: &Path, login_page: &str) -> Option<(Tokens, String)> {
    let mut event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Riot Login")
        .with_inner_size(LogicalSize::new(800, 950))
        .with_resizable(false)
        .build(&event_loop)
        .expect("window should create");

    let handle = window.window_handle().expect("window should have handle");
    let handle = match handle.as_raw() {
        RawWindowHandle::Win32(handle) => handle,
        it => panic!("Unexpected handle variant {it:?}"),
    };

    let hwnd = handle.hwnd.get() as HWND;

    let controller: Rc<OnceCell<Controller>> = Rc::new(OnceCell::new());
    let controller_clone = controller.clone();

    let uri: Rc<OnceCell<String>> = Rc::new(OnceCell::new());
    let uri_clone = uri.clone();

    let cookie_cell: Rc<OnceCell<String>> = Rc::new(OnceCell::new());
    let cookie_cell_2 = cookie_cell.clone();

    let event_loop_proxy = event_loop.create_proxy();
    let initial_page = login_page.to_string();
    Environment::builder()
        .with_user_data_folder(&profile_folder)
        .build(move |env| {
            let env = env?;
            let _env = env.create_controller(hwnd, move |c| {
                let c = c?;

                let webview = c.get_webview()?;
                webview
                    .get_webview2()?
                    .get_cookie_manager()?
                    .delete_all_cookies()?;

                unsafe {
                    let mut rect = mem::zeroed();
                    GetClientRect(hwnd, &mut rect);
                    c.put_bounds(rect)?;
                }

                webview.add_navigation_starting(move |webview, event| {
                    let uri = event.get_uri()?;

                    if uri.contains("access_token") {
                        event.put_cancel(true)?;
                        uri_clone.set(uri).expect("set uri failed");

                        let webview2 = webview.get_webview2()?;
                        let cookie_manager = webview2.get_cookie_manager()?;

                        let event_loop_proxy_2 = event_loop_proxy.clone();
                        let cookie_cell_3 = cookie_cell_2.clone();
                        cookie_manager
                            .get_cookies(
                                "https://auth.riotgames.com/authorize",
                                move |cookie_list| {
                                    let mut cookies = Vec::new();

                                    let count = cookie_list.get_count()?;
                                    for i in 0..count {
                                        let cookie = cookie_list.get_value_at_index(i)?;
                                        cookies.push(cookie);
                                    }

                                    let cookie_str = cookies
                                        .iter()
                                        .filter_map(|c| {
                                            Some(format!(
                                                "{}={}",
                                                c.get_name().ok()?,
                                                c.get_value().ok()?
                                            ))
                                        })
                                        .collect::<Vec<_>>()
                                        .join("; ")
                                        .to_string();

                                    cookie_cell_3
                                        .set(cookie_str)
                                        .expect("cookies should be set");

                                    event_loop_proxy_2
                                        .send_event(())
                                        .map_err(|_| webview2::Error::new(E_FAIL))?;

                                    Ok(())
                                },
                            )
                            .expect("failed to load cookies");
                    }

                    Ok(())
                })?;
                webview.navigate(&initial_page)?;
                controller_clone.set(c).expect("set controller cell");

                Ok(())
            });

            Ok(())
        })
        .expect("failed to create environment");

    let exit_code = event_loop.run_return(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::ExitWithCode(1),
        Event::UserEvent(_) => *control_flow = ControlFlow::Exit,
        _ => *control_flow = ControlFlow::Wait,
    });

    println!("Window exited with exit code {exit_code}");

    let tokens = uri.get().expect("uri should have been set");
    let cookies = cookie_cell
        .get()
        .expect("cookies should have been set")
        .clone();

    let tokens = url::Url::parse(&tokens)
        .ok()
        .and_then(|u| u.fragment().map(|str| str.to_string()))
        .map(|str| serde_urlencoded::from_str(&str).expect("fragment should contain tokens"))
        .unwrap();

    Some((tokens, cookies))
}
