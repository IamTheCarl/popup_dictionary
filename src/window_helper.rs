use std::error::Error;
use x11rb::{
    connection::Connection,
    protocol::xproto::{AtomEnum, ConfigureWindowAux, ConnectionExt, Window},
    rust_connection::RustConnection,
};

pub fn move_window_x11(x: i32, y: i32) -> Result<(), Box<dyn Error>> {
    let (connection, display_idx) = RustConnection::connect(None)?;
    let display = &connection.setup().roots[display_idx];

    println!("Looking for x11 window.");
    match find_window_by_title(&connection, display.root, crate::app::APP_NAME)? {
        Some(window) => {
            println!("Found window: 0x{:x}", window);
            move_window(&connection, window, x, y)?;
            println!("Moved window to position ({}, {})", x, y);
        }
        None => {
            println!("Window not found!");
        }
    }

    Ok(())
}

fn find_window_by_title(
    connection: &RustConnection,
    root: Window,
    title: &str,
) -> Result<Option<Window>, Box<dyn Error>> {
    let tree = connection.query_tree(root)?.reply()?;

    for &child in &tree.children {
        let window_title = get_window_title(connection, child)?;
        if window_title.contains(title) {
            return Ok(Some(child));
        }

        if let Some(found) = find_window_by_title(connection, child, title)? {
            return Ok(Some(found));
        }
    }

    Ok(None)
}

fn get_window_title(connection: &RustConnection, window: Window) -> Result<String, Box<dyn Error>> {
    let net_wm_name = connection
        .intern_atom(false, b"_NET_WM_NAME")?
        .reply()?
        .atom;
    let utf8_string = connection.intern_atom(false, b"UTF8_STRING")?.reply()?.atom;

    if let Ok(reply) = connection
        .get_property(false, window, net_wm_name, utf8_string, 0, 1024)?
        .reply()
    {
        if !reply.value.is_empty() {
            return Ok(String::from_utf8_lossy(&reply.value).into_owned());
        }
    }

    // Fallback to WM_NAME
    if let Ok(reply) = connection
        .get_property(false, window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 1024)?
        .reply()
    {
        if !reply.value.is_empty() {
            return Ok(String::from_utf8_lossy(&reply.value).into_owned());
        }
    }

    Ok(String::new())
}

fn move_window(
    connection: &RustConnection,
    window: Window,
    x: i32,
    y: i32,
) -> Result<(), Box<dyn Error>> {
    let values = ConfigureWindowAux::new().x(x).y(y);

    connection.configure_window(window, &values)?;
    connection.flush()?;

    Ok(())
}
