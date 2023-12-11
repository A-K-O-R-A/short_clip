// we import the necessary modules (only the core X module in this application).
use xcb::x;
// we need to import the `Xid` trait for the `resource_id` call down there.
use xcb::x::ModMask;
// Many xcb functions return a `xcb::Result` or compatible result.

mod upload;
use upload::handle_hotkey;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the X server.
    let (conn, screen_num) = xcb::Connection::connect(None)?;

    // Fetch the `x::Setup` and get the main `x::Screen` object.
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();

    let cookie = conn.send_request_checked(&x::GrabKey {
        owner_events: true,
        grab_window: screen.root(),
        modifiers: (ModMask::CONTROL | ModMask::N2),
        key: 30,
        pointer_mode: x::GrabMode::Async,
        keyboard_mode: x::GrabMode::Async,
    });
    // We now check if the window creation worked.
    // A cookie can't be cloned; it is moved to the function.
    conn.check_request(cookie)?;

    // A second time to also catch the key events without mod 2
    let cookie = conn.send_request_checked(&x::GrabKey {
        owner_events: true,
        grab_window: screen.root(),
        modifiers: (ModMask::CONTROL),
        key: 30,
        pointer_mode: x::GrabMode::Async,
        keyboard_mode: x::GrabMode::Async,
    });
    // We now check if the window creation worked.
    // A cookie can't be cloned; it is moved to the function.
    conn.check_request(cookie)?;

    // We enter the main event loop
    loop {
        match conn.wait_for_event()? {
            xcb::Event::X(x::Event::KeyPress(_ev)) => {
                handle_hotkey()?;

                // The Strg + U Key was pressed
                conn.flush()?;
            }
            xcb::Event::X(x::Event::ClientMessage(ev)) => {
                // We have received a message from the server
                if let x::ClientMessageData::Data32([_atom, ..]) = ev.data() {}
            }
            _ => {}
        }
    }
}
