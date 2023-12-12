use std::{path::PathBuf, time::Duration};

use arboard::Clipboard;
use image::codecs::png::PngEncoder;

use crate::{sys::fs::guess_path_content, upload::ClipboardContent};

pub fn set_clipboard(string: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(string)?;

    // Otherwise clipboard contents will be dropped immediately
    std::thread::sleep(Duration::from_millis(100));

    Ok(())
}

pub fn read_clipboard() -> Result<ClipboardContent, Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;

    let content: ClipboardContent;

    if let Ok(image_data) = clipboard.get_image() {
        let content_type = "image/png".to_owned();

        // Load Bitmap Image
        let img_buf = image::RgbaImage::from_raw(
            image_data.width as u32,
            image_data.height as u32,
            image_data.bytes.to_vec(),
        )
        .unwrap();
        let mut v = Vec::with_capacity(image_data.bytes.len());

        // Convert to png for upload
        img_buf.write_with_encoder(PngEncoder::new(&mut v))?;

        content = ClipboardContent {
            content_type,
            data: v,
        };
    } else if let Ok(clipboard_content) = clipboard.get_text() {
        let content_type;

        if clipboard_content.starts_with("file://") {
            let path = PathBuf::from(&clipboard_content[7..]);
            path.try_exists()?;

            content_type = guess_path_content(&path);

            let data = std::fs::read(path)?;

            content = ClipboardContent { content_type, data };
        } else {
            // Check if the content is a valid url
            if let Ok(_) = url::Url::parse(&clipboard_content) {
                // Let the backend know that this is a url
                content_type = "text/uri-list".to_owned();
            } else {
                content_type = "text/plain".to_owned();
            }

            content = ClipboardContent {
                content_type,
                data: clipboard_content.into_bytes(),
            };
        }
    } else {
        // No content in clipboard
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Clipboard empty",
        )));
    }

    Ok(content)
}
