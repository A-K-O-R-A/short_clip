use std::path::PathBuf;

use clipboard_win::formats;
use clipboard_win::get_clipboard;
use image::codecs::png::PngEncoder;
use image::ImageFormat;

use crate::sys::fs::guess_path_content;
use crate::upload::ClipboardContent;

pub fn set_clipboard(string: &str) -> Result<(), Box<dyn std::error::Error>> {
    match clipboard_win::set_clipboard_string(string) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))),
    }
}

pub fn read_clipboard() -> Result<ClipboardContent, Box<dyn std::error::Error>> {
    let content: ClipboardContent;

    if let Ok(bitmap) = get_clipboard(formats::Bitmap) {
        // Load Bitmap Image
        let dyn_img = image::load_from_memory_with_format(&bitmap, ImageFormat::Bmp)?;
        let mut v = Vec::with_capacity(bitmap.len());

        // Convert to png for upload
        dyn_img.write_with_encoder(PngEncoder::new(&mut v))?;

        content = ClipboardContent {
            content_type: "image/png".to_owned(),
            data: v,
        }
    } else if let Ok(file_list) = get_clipboard(formats::FileList) {
        // Filter valid paths
        let paths: Vec<PathBuf> = file_list
            .into_iter()
            .map(|f| PathBuf::from(f))
            .filter(|p| p.try_exists().unwrap_or(false))
            .collect();

        if paths.len() == 1 {
            let path = &paths[0];

            let content_type = guess_path_content(path);
            let data = std::fs::read(path)?;

            content = ClipboardContent {
                content_type: content_type,
                data: data,
            }
        } else if paths.len() >= 1 {
            // TODO: Implement file zipping
            // For now multiple files will be ignored and only the first one gets uploaded
            let path = &paths[0];

            let content_type = guess_path_content(path);
            let data = std::fs::read(path)?;

            content = ClipboardContent {
                content_type: content_type,
                data: data,
            }
        } else {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Trying to copy 0 files",
            )));
        }
    } else if let Ok(text) = get_clipboard::<String, _>(formats::Unicode) {
        content = ClipboardContent {
            content_type: "text/plain".to_owned(),
            data: text.into_bytes(),
        }
    } else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Clipboard empty",
        )));
    }

    Ok(content)
}
