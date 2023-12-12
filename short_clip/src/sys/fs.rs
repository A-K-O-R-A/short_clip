use std::{
    fs::{DirEntry, File},
    io::{Read, Seek, Write},
    path::Path,
};

use zip::write::FileOptions;

pub fn guess_path_content(path: &Path) -> String {
    if let Some(ext) = path.extension() {
        let guess = mime_guess::from_ext(ext.to_str().unwrap()).first();

        match guess {
            Some(g) => g.to_string(),
            None => "application/octet-stream".to_owned(),
        }
    } else {
        "application/octet-stream".to_owned()
    }
}

/// Copie from https://github.com/zip-rs/zip/blob/3e88fe66c941d411cff5cf49778ba08c2ed93801/examples/write_dir.rs#L65
#[allow(dead_code)]
fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {path:?} as {name:?} ...");
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!("adding dir {path:?} as {name:?} ...");
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}
