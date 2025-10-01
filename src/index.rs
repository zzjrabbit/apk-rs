use std::{collections::BTreeMap, io::BufRead};

pub struct PackageIndex {
    infos: BTreeMap<String, PackageInfo>,
}

impl PackageIndex {
    pub fn new<R>(mut reader: R) -> Result<Self, std::io::Error>
    where
        R: BufRead,
    {
        let mut infos = BTreeMap::new();

        loop {
            match PackageInfo::parse(&mut reader) {
                Ok(info) => match info {
                    Some(info) => infos.insert(info.name.clone(), info),
                    None => break Ok(Self { infos }),
                },
                Err(err) => return Err(err),
            };
        }
    }

    pub fn get(&self, name: &str) -> Option<&PackageInfo> {
        self.infos.get(name)
    }
}

pub struct PackageInfo {
    name: String,
    version: String,
}

impl PackageInfo {
    fn parse<R>(mut reader: R) -> Result<Option<Self>, std::io::Error>
    where
        R: BufRead,
    {
        let mut buffer = String::new();

        let mut package = None;
        let mut version = None;

        loop {
            buffer.clear();
            reader.read_line(&mut buffer)?;

            if buffer.is_empty() {
                break Ok(None);
            }

            if buffer.starts_with("P:") {
                package = Some(
                    buffer
                        .strip_suffix("\n")
                        .unwrap()
                        .trim_start_matches("P:")
                        .to_string(),
                );
            } else if buffer.starts_with("V:") {
                version = Some(
                    buffer
                        .strip_suffix("\n")
                        .unwrap()
                        .trim_start_matches("V:")
                        .to_string(),
                );
            }

            if package.is_some() && version.is_some() {
                break Ok(Some(Self {
                    name: package.unwrap(),
                    version: version.unwrap(),
                }));
            }
        }
    }

    pub fn file_name(&self) -> String {
        format!("{}-{}.apk", self.name, self.version)
    }
}
