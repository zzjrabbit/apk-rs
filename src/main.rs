use std::{
    collections::{BTreeMap, btree_map::Entry},
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Read},
    path::PathBuf,
};

use path_slash::PathBufExt;
use rayon::prelude::*;

use crate::{builder::build_img, config::Config, download::download_file, index::PackageIndex};

mod builder;
mod config;
mod download;
mod index;

fn main() {
    let mut config_file = File::open("apk_rs.toml").unwrap();

    let mut config = Vec::new();

    config_file.read_to_end(&mut config).unwrap();

    let config: Config = toml::from_slice(&config).unwrap();

    let _ = std::fs::create_dir("target");
    let _ = std::fs::create_dir("target/apk");

    let apk_index_path = PathBuf::from("target/apk/apkindex");
    let image_dest = PathBuf::from("target/apk/image");
    let package_dir = PathBuf::from("target/apk/package");

    if !std::fs::exists(&apk_index_path).unwrap() {
        download::download_file(
            "APKINDEX.tar.gz",
            &apk_index_path,
            false,
            config.package.mirror.clone(),
        );
    }

    let index = apk_index_path.join("APKINDEX");
    let index_file = File::open(&index).unwrap();

    let package_index = PackageIndex::new(BufReader::new(index_file)).unwrap();

    let _ = fs::create_dir(&image_dest);

    let mut files = BTreeMap::new();

    let data = config
        .package
        .list
        .par_iter()
        .map(|package_name| {
            let mut files = BTreeMap::new();

            let package = package_index.get(package_name).unwrap();

            let package_dir = package_dir.join(package_name);

            download_file(
                &package.file_name(),
                &package_dir,
                true,
                config.package.mirror.clone(),
            );

            let src_abs = package_dir.canonicalize().unwrap();

            let dest_base = PathBuf::from("");
            let walker = walkdir::WalkDir::new(&src_abs)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file() || e.file_type().is_symlink());

            for entry in walker {
                if !filter_file_names(entry.file_name().to_str().unwrap()) {
                    continue;
                }

                let source = entry.path();
                let rel_path = source.strip_prefix(&src_abs).unwrap();

                let dest = dest_base.join(rel_path);
                let dest_str = dest.to_slash().expect("Invalid UTF-8 path");

                if let Entry::Vacant(e) = files.entry(dest_str.to_string()) {
                    e.insert(source.to_path_buf());
                } else {
                    println!("Skipping duplicate file: '{}'", dest_str);
                }
            }

            files
        })
        .collect::<Vec<BTreeMap<String, PathBuf>>>();

    for sub_files in data {
        files.extend(sub_files);
    }

    if let Some(cfiles) = config.files {
        for file in cfiles {
            let source = PathBuf::from(file.source);
            let dest = file.dest;

            if fs::exists(source.clone()).is_err() {
                continue;
            }

            files.insert(dest, source);
        }
    }

    println!("files: {:?}", files);

    if let Some(image) = config.image {
        build_img(files, &PathBuf::from(image.path)).unwrap();
    } else {
        let root = PathBuf::from(config.root.unwrap_or("/".to_string()));

        for (dest, source) in files {
            let dest = root.join(dest);

            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }

            println!("source: {:?} dest: {:?}", source.display(), dest.display());

            let source = File::open(source).unwrap();
            let dest = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(dest)
                .unwrap();

            let mut reader = BufReader::new(source);
            let mut writer = BufWriter::new(dest);

            std::io::copy(&mut reader, &mut writer).unwrap();
        }
    }
}

fn filter_file_names(name: &str) -> bool {
    const IGNORED_FILES: &[&str] = &[
        ".pre-deinstall",
        ".post-deinstall",
        ".pre-upgrade",
        ".post-upgrade",
        ".pre-install",
        ".post-install",
        ".PKGINFO",
    ];
    const IGNORED_PREFIXES: &[&str] = &[".SIGN"];

    if IGNORED_FILES.contains(&name) {
        return false;
    }

    if IGNORED_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
    {
        return false;
    }

    true
}
