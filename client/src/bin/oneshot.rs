use short_clip_client::upload::upload_clipboard;

/// When executed this will upload the current clipboard content
/// and replace it with a link
fn main() -> Result<(), Box<dyn std::error::Error>> {
    upload_clipboard()
}
