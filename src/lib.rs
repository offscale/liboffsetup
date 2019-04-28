use std::path::PathBuf;

use structopt::StructOpt;

// By using structopt, we can use the same structure from CLI, config file, and internally.

#[derive(StructOpt, Debug)]
#[structopt(name = "offsetup")]
struct OffSetup {
    name: String,
    version: String,

    platforms: Option<Vec<Platform>>,
    exposes: Option<Exposes>,

    /// Activate debug mode
    #[structopt(short = "d", long = "debug")]
    debug: bool,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "platform")]
struct Platform {
    versions: Vec<String>,

    arch: Option<String>,

    // TODO: Force `download_directory` to be required if `download` specified
    download_directory: Option<String>,
    download: Option<Download>,

    // Because Rust, we'll need to explicitly write each here
    apt: Option<Vec<String>>,
    yum: Option<Vec<String>>,
    ports: Option<Vec<String>>,
    brew: Option<Vec<String>>,
    choco: Option<Vec<String>>,

    install_priority: Vec<String>,
    skip_install: Option<bool>,
    fail_silently: Option<bool>,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "download")]
struct Download {
    uri: String,
    // TODO: parse to Uri
    hash_protocol: Option<String>,
    // TODO: use Digest trait somehow, and include sha512 default
    hash: String,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "exposes")]
struct Exposes {
    ports: Option<Ports>,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "ports")]
struct Ports {
    udp: Option<Vec<u16>>,
    tcp: Option<Vec<u16>>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
