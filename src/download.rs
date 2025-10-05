use flate2::bufread::GzDecoder;
use std::{
    io::{BufReader, Seek, SeekFrom},
    path::Path,
};
use tar::Archive;
use tempfile::NamedTempFile;

use ureq::get;

fn get_file_url(file_name: &str, mirror: Option<String>) -> String {
    const APK_URL: &str = "https://dl-cdn.alpinelinux.org/edge/main/x86_64";
    format!("{}/{}", mirror.unwrap_or(APK_URL.to_string()), file_name)
}

pub fn download_file(file_name: &str, dest: &Path, package: bool, mirror: Option<String>) {
    let url = get_file_url(file_name, mirror);

    let response = get(&url).call().unwrap();
    let body = response.into_body();
    let mut reader = body.into_reader();

    let mut tempfile = NamedTempFile::new().unwrap();
    std::io::copy(&mut reader, &mut tempfile).unwrap();

    let mut tempfile = tempfile.into_file();
    tempfile.seek(SeekFrom::Start(0)).unwrap();

    let mut reader = BufReader::new(tempfile);

    for _ in 0..2 {
        let mut tar = GzDecoder::new(reader);
        let mut archive = Archive::new(&mut tar);

        archive.unpack(dest).unwrap();
        reader = tar.into_inner();
    }

    if package {
        let tar = GzDecoder::new(reader);
        let mut archive = Archive::new(tar);
        archive.unpack(dest).unwrap();
    }
}
