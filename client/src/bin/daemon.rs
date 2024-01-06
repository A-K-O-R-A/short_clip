use short_clip_client::sys;
use short_clip_client::upload;
use upload::upload_clipboard;

/// This will create a long running process that grabs the
/// key combination Ctrl + U and uploads the clipboard
fn main() -> Result<(), Box<dyn std::error::Error>> {
    sys::hotkey::create_listener(upload_clipboard)
}
