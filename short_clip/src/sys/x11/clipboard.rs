use arboard::{Clipboard, ImageData};

pub fn set_clipboard(string: &str) -> Result<(), Box<dyn std::error::Error>> {
    clipboard.set_text(link)?;

    // Otherwise clipboard contents will be dropped immediately
    std::thread::sleep(Duration::from_millis(100));
}

pub fn read_clipboard() -> Result<ClipboardContent, Box<dyn std::error::Error>> {
    let mut clipboard = Clipboard::new()?;

    let content: ClipboardContent;

    if let Ok(image_data) = clipboard.get_image() {
        let content_type = "image/png".to_owned();
        let data = image_data_to_png(image_data)?;

        content = ClipboardContent { content_type, data };
    } else if let Ok(content) = clipboard.get_text() {
        let content_type;
        // println!("Clipboard text was: {}", content);

        if content.starts_with("file://") {
            let path = PathBuf::from(&content[7..]);
            path.try_exists()?;

            if let Some(ext) = path.extension() {
                let guess = mime_guess::from_ext(ext.to_str().unwrap()).first();

                content_type = match guess {
                    Some(g) => g.to_string(),
                    None => "application/octet-stream".to_owned(),
                };
            } else {
                content_type = "application/octet-stream".to_owned();
            }

            let data = std::fs::read(path)?;

            content = ClipboardContent { content_type, data };
        } else {
            // Check if the content is a valid url
            if let Ok(_) = url::Url::parse(&content) {
                // Let the backend know that this is a url
                content_type = "text/uri-list".to_owned();
            } else {
                content_type = "text/plain".to_owned();
            }

            content = ClipboardContent {
                content_type,
                data: content.into_bytes(),
            };
        }
    } else {
        println!("Clipboard empty");
        // No content in clipboard
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Clipboard empty",
        )));
    }

    Ok(content)
}

fn image_data_to_png(image_data: ImageData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut w = Vec::with_capacity(image_data.bytes.len());
    let mut encoder = png::Encoder::new(&mut w, image_data.width as u32, image_data.height as u32); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2)); // 1.0 / 2.2, unscaled, but rounded
    let source_chromaticities = png::SourceChromaticities::new(
        // Using unscaled instantiation here
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000),
    );
    encoder.set_source_chromaticities(source_chromaticities);

    let mut writer = encoder.write_header()?;
    writer.write_image_data(&image_data.bytes)?;
    writer.finish()?;

    Ok(w)
}
