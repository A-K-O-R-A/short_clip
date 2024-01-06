use crate::{
    sys::clipboard::{read_clipboard, set_clipboard},
    CONFIG,
};

pub struct ClipboardContent {
    pub content_type: String,
    pub data: Vec<u8>,
}

pub fn handle_hotkey() -> Result<(), Box<dyn std::error::Error>> {
    let content = read_clipboard()?;

    if let Some(link) = upload_contents(&content.data, &content.content_type) {
        set_clipboard(&link)?;
    } else {
        eprintln!("Uploading failed");
    }

    Ok(())
}

fn upload_contents(data: &[u8], content_type: &str) -> Option<String> {
    let config = CONFIG.get().unwrap();

    let result = ureq::post(&config.host)
        .set("authorization", &config.token)
        .set("Content-Type", content_type)
        .send_bytes(data);

    match result {
        Ok(resp) => {
            if resp.status() != 201 {
                eprintln!("{}", resp.status_text());
                return None;
            }

            resp.header("Location").map(|s| s.to_owned())
        }
        Err(e) => {
            eprintln!("{e}");
            return None;
        }
    }
}
