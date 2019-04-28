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

    // Linux
    apt: Option<Vec<String>>,
    aptitude: Option<Vec<String>>,
    // https://wiki.sabayon.org/index.php?title=En:Entropy
    equo: Option<Vec<String>>,
    // https://wiki.gentoo.org/wiki/Handbook:AMD64/Working/Portage
    emerge: Option<Vec<String>>,
    // https://flathub.org
    flatpak: Option<Vec<String>>,
    // https://www.gnu.org/software/guix/
    guix: Option<Vec<String>>,
    // https://nixos.org/nix/manual/#chap-quick-start
    nix: Option<Vec<String>>,
    // http://www.openpkg.org/documentation/tutorial/
    openpkg: Option<Vec<String>>,
    // http://wiki.openmoko.org/wiki/Opkg
    opkg: Option<Vec<String>>,
    // https://wiki.archlinux.org/index.php/Pacman
    pacman: Option<Vec<String>>,
    // https://puppylinux.org/wikka/ppm
    ppm: Option<Vec<String>>,
    // https://github.com/examachine/pisi
    pisi: Option<Vec<String>>,

    // http://yum.baseurl.org
    yum: Option<Vec<String>>,
    // https://rpm-software-management.github.io
    dnf: Option<Vec<String>>,
    // http://rpmfind.net/linux/rpm2html/search.php?query=up2date
    up2date: Option<Vec<String>>,

    apk: Option<Vec<String>>,
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
