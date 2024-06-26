mod channel;

use std::{
    collections::BTreeMap,
    fmt::Write,
    io::{BufRead, BufReader},
    time::SystemTime,
};

use mdbook::{
    book::{Book, BookItem},
    errors::Error,
    preprocess::{Preprocessor, PreprocessorContext},
};

const CHANNELS: &[&str] = &["stable", "beta", "nightly"];
const CHANNEL_URL_PREFIX: &str = "https://static.rust-lang.org/dist/channel-rust-";
const RUSTUP_URLS: &str =
    "https://raw.githubusercontent.com/rust-lang/rustup.rs/stable/ci/cloudfront-invalidation.txt";

/// A representation of a rust target platform. `stable`, `beta`, and `nightly`
/// represent whether the platform is available on that channel. `stable` also
/// carries the specific stable compiler version.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Platform {
    stable: Option<String>,
    beta: bool,
    nightly: bool,
}

impl Default for Platform {
    fn default() -> Self {
        Self {
            stable: None,
            beta: false,
            nightly: false,
        }
    }
}

/// `Blacksmith` builds dynamic tables and lists to display in the Rust Forge.
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Blacksmith {
    last_update: Option<u64>,
    rustup: Vec<String>,
    stable_version: Option<String>,
    platforms: BTreeMap<String, Platform>,
}

impl Blacksmith {
    /// Creates a new instance of `Blacksmith`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the data in this `Blacksmith` instance was not updated within the TTL.
    pub fn is_stale(&self, ttl: u64) -> bool {
        if let (Some(last_update), Some(now)) = (self.last_update, unix_time()) {
            last_update + ttl < now
        } else {
            true
        }
    }

    /// Populates a `Blacksmith` instance with data gathered from Rust's CI and
    /// distribution channels.
    pub fn init() -> Result<Self, Box<dyn std::error::Error>> {
        let mut blacksmith = Self::new();

        let rustup_url_regex =
            regex::Regex::new(r"^rustup/dist/([^/]+)/rustup-init(?:\.exe)?$").unwrap();
        for line in BufReader::new(reqwest::blocking::get(RUSTUP_URLS)?).lines() {
            if let Some(m) = rustup_url_regex.captures(&(line?)) {
                blacksmith
                    .rustup
                    .push(m.get(1).unwrap().as_str().to_string());
            }
        }
        log::info!("Found {} targets for rustup", blacksmith.rustup.len());

        for &channel_name in CHANNELS {
            let channel_url = format!("{}{}.toml", CHANNEL_URL_PREFIX, channel_name);
            let content = reqwest::blocking::get(&channel_url)?.text()?;
            let rust = toml::from_str::<crate::channel::Channel>(&content)?
                .pkg
                .rust;
            log::info!(
                "Found {} targets for {} channel (v{})",
                rust.target.len(),
                channel_name,
                rust.version
            );

            let vers = rust.version.split(' ').next().unwrap().to_string();
            let platforms = rust
                .target
                .into_iter()
                .filter_map(|(target, content)| {
                    if content.available {
                        Some(target)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            if channel_name == "stable" {
                blacksmith.stable_version = Some(vers.clone());
            }

            for platform in platforms {
                let entry = blacksmith
                    .platforms
                    .entry(platform)
                    .or_insert_with(Platform::default);

                match channel_name {
                    "stable" => entry.stable = Some(vers.clone()),
                    "beta" => entry.beta = true,
                    "nightly" => entry.nightly = true,
                    _ => unreachable!(),
                }
            }
        }

        blacksmith.last_update = unix_time();
        Ok(blacksmith)
    }

    /// Creates a list of hyperlinks to `rustup-init` based on what targets
    /// rustup provided using the following URL schema. Where `target` is the
    /// platforms target tuple (`x86_64-apple-darwin`) and `suffix` is a target
    /// specific file extension.
    /// ```url
    /// https://static.rust-lang.org/rustup/dist/{target}/rustup-init{suffix}
    /// ```
    fn generate_rustup_init_list(&self) -> String {
        let mut buffer = String::new();

        for target in &self.rustup {
            let suffix = if target.contains("windows") {
                ".exe"
            } else {
                ""
            };

            writeln!(
                buffer,
                "- [{target}](https://static.rust-lang.org/rustup/dist/{target}/rustup-init{suffix})",
                target = target,
                suffix = suffix,
            ).unwrap();
        }

        buffer
    }

    /// Generates a table of links to the standalone installer packages for
    /// each platform.
    fn generate_standalone_installers_table(&self) -> String {
        let mut buffer = String::new();
        writeln!(
            buffer,
            "platform | stable ({}) | beta | nightly",
            self.stable_version.as_ref().unwrap()
        )
        .unwrap();

        writeln!(buffer, "---------|--------|------|--------").unwrap();

        for (name, platform) in &self.platforms {
            let extension = if name.contains("windows") {
                "msi"
            } else if name.contains("darwin") {
                "pkg"
            } else {
                "tar.xz"
            };

            let stable_links = platform
                .stable
                .as_ref()
                .map(|version| generate_standalone_links("rust", version, name, extension))
                .unwrap_or_else(String::new);

            let beta_links = if platform.beta {
                generate_standalone_links("rust", "beta", name, extension)
            } else {
                String::new()
            };

            let nightly_links = if platform.nightly {
                generate_standalone_links("rust", "nightly", name, extension)
            } else {
                String::new()
            };

            writeln!(
                buffer,
                "`{name}` | {stable} | {beta} | {nightly}",
                name = name,
                stable = stable_links,
                beta = beta_links,
                nightly = nightly_links
            )
            .unwrap();
        }

        buffer
    }

    /// Generates a similar table to `generate_standalone_installers_table`
    /// except for the rust source code packages.
    fn generate_source_code_table(&self) -> String {
        let mut buffer = String::new();

        writeln!(buffer, "Channel | Archives + Signatures").unwrap();
        writeln!(buffer, "--------|----------------------").unwrap();

        for &channel in CHANNELS {
            if channel == "stable" {
                let stable_version = self.stable_version.as_ref().unwrap();
                writeln!(
                    buffer,
                    "stable ({}) | {}",
                    stable_version,
                    generate_standalone_links("rustc", stable_version, "src", "tar.xz")
                )
                .unwrap();
            } else {
                writeln!(
                    buffer,
                    "{} | {}",
                    channel,
                    generate_standalone_links("rustc", &channel, "src", "tar.xz")
                )
                .unwrap();
            }
        }

        buffer
    }
}

/// Generates links to standalone installers provided a rust version or channel,
/// target name, and file extension.
fn generate_standalone_links(base: &str, stem: &str, name: &str, extension: &str) -> String {
    let url = format!(
        "https://static.rust-lang.org/dist/{base}-{stem}-{name}.{extension}",
        extension = extension,
        name = name,
        stem = stem,
        base = base,
    );

    format!(
        "[{extension}]({url}) <br> [{extension}.asc]({url}.asc)",
        extension = extension,
        url = url,
    )
}

fn unix_time() -> Option<u64> {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

impl Preprocessor for Blacksmith {
    fn name(&self) -> &str {
        "blacksmith"
    }

    fn run(&self, _: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        fn recursive_replace(book_item: &mut BookItem, old: &str, new: &str) {
            let chapter = match book_item {
                BookItem::Chapter(chapter) => chapter,
                _ => return,
            };

            chapter.content = chapter.content.replace(old, new);

            for sub_item in &mut chapter.sub_items {
                recursive_replace(sub_item, old, new);
            }
        }

        let rustup_init_list = self.generate_rustup_init_list();
        let standalone_installers_table = self.generate_standalone_installers_table();
        let source_code_table = self.generate_source_code_table();

        // TODO: Currently we're performing a global search for any of the
        // variables as that's the most flexible for adding more dynamic
        // content, and the time to traverse is fast enough to not be noticable.
        // However if the processing time begins to become a bottleneck this
        // should change.
        for item in &mut book.sections {
            recursive_replace(item, "{{#rustup_init_list}}", &rustup_init_list);
            recursive_replace(item, "{{#installer_table}}", &standalone_installers_table);
            recursive_replace(item, "{{#source_code_table}}", &source_code_table);
        }

        Ok(book)
    }

    /// `Blacksmith`'s operations are renderer independent as they output
    /// markdown.
    fn supports_renderer(&self, _renderer: &str) -> bool {
        true
    }
}
