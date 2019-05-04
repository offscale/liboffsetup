use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::string::ParseError;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use structopt::StructOpt;

#[derive(Debug, Deserialize)]
struct VecOfU16 {
    data: Vec<u16>,
}

impl FromStr for VecOfU16 {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            data: input
                .trim()
                .split(',')
                .map(|s| s.parse().unwrap())
                .collect(),
        })
    }
}

// Since structopt/clap does not support config file, only cli and env, we split the two between
// 1) config for file and environment
// 2) structopt for CLI
#[derive(Debug, Deserialize)]
pub struct OffSetup {
    name: String,
    version: String,

    dependencies: Option<Dependencies>,
    exposes: Option<Exposes>,
}

#[derive(StructOpt, Debug, Deserialize)]
#[structopt(name = "offsetup")]
pub struct OffSetupCli {
    /// Activate debug mode
    #[structopt(short = "d", long = "debug", env = "OFFSETUP_DEBUG")]
    debug: bool,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(
        short = "v",
        long = "verbose",
        parse(from_occurrences),
        env = "OFFSETUP_VERBOSITY"
    )]
    verbose: u8,
}

#[derive(Debug, Deserialize)]
struct System {
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
}

#[derive(Debug, Deserialize)]
struct Dependencies {
    applications: HashMap<String, Application>,
    platforms: HashMap<String, Platform>,
}

#[derive(Debug, Deserialize)]
struct Application {
    pkg: Option<String>,
    version: Option<String>,
    env: Option<String>,

    install_priority: Option<Vec<String>>,
    skip_install: Option<bool>,
    fail_silently: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Platform {
    versions: Vec<String>,

    arch: Option<String>,

    source: Option<Source>,

    system: Option<HashMap<String, System>>,

    pre_install: Option<Vec<String>>,
    install_priority: Option<Vec<String>>,
    skip_install: Option<bool>,
    fail_silently: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Source {
    // TODO: Force `download_directory` to be required if `download` specified
    download_directory: Option<String>,
    download: Option<Download>,

    system: Option<HashMap<String, System>>,
}

#[derive(Debug, Deserialize)]
struct Download {
    uri: String,
    // TODO: parse to Uri
    hash_protocol: Option<String>,
    // TODO: use Digest trait somehow, and include sha512 default
    hash: String,
}

#[derive(Debug, Deserialize)]
enum Exposes {
    Ports {
        tcp: Option<Vec<u16>>,
        udp: Option<Vec<u16>>,
    },
}

impl OffSetup {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Start off by merging in the "default" configuration file
        s.merge(File::with_name("config/default"))?;

        // Add in the current environment file
        // Default to 'development' env
        // Note that this file is _optional_
        let env = env::var("RUN_MODE").unwrap_or("development".into());
        s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // Add in a local configuration file
        // This file shouldn't be checked in to git
        s.merge(File::with_name("config/local").required(false))?;

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(Environment::with_prefix("app"))?;

        // You may also programmatically change settings
        s.set("database.url", "postgres://")?;

        // Now that we're done, let's access our configuration
        println!("debug: {:?}", s.get_bool("debug"));
        println!("database: {:?}", s.get::<String>("database.url"));

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_use_cli() {
        let args = OffSetupCli::from_args();
        println!("{:?}", args);
    }

    #[test]
    fn can_read_simple_ports() {
        println!("testing ports...");
        let f = || -> Result<Exposes, ConfigError> {
            let mut s = Config::new();

            s.merge(File::with_name("examples/exposes"))?;
            println!("merged: {:?}", s);

            match s.get::<Option<Vec<u16>>>("ports.tcp") {
                Ok(udp) => assert!(udp.is_some()),
                Err(e) => panic!(format!("error getting tdp: {:?}", e)),
            }

            s.try_into()
        };
        match f() {
            Ok(s) => println!("Successful: {:?}", s),
            Err(e) => panic!(format!("Failed to get configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_read_simple_file() {
        let f = || -> Result<OffSetup, ConfigError> {
            let mut s = Config::new();

            s.merge(File::with_name("examples/simple"))?;
            println!("merged: {:?}", s);

            match s.get::<Option<Vec<u16>>>("exposes.ports.tcp") {
                Ok(tcp) => assert!(tcp.is_some()),
                Err(e) => panic!(format!("error getting tcp: {:?}", e)),
            }

            s.try_into()
        };
        match f() {
            Ok(s) => println!("Successful: {:?}", s),
            Err(e) => panic!(format!("Failed to get configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_read_redis_file() {
        let f = || -> Result<OffSetup, ConfigError> {
            let mut s = Config::new();

            s.merge(File::with_name("examples/redis"))?;
            println!("merged: {:?}", s);

            println!(
                "platforms: {:?}",
                s.get::<Option<HashMap<String, Platform>>>("dependencies.platforms")
            );

            match s.get::<Platform>("dependencies.platforms.windows") {
                Ok(windows) => {
                    assert!(windows.arch.is_some());
                    assert_eq!(windows.arch.unwrap(), "x86_64")
                }
                Err(e) => panic!(format!("error getting windows platform: {:?}", e)),
            }

            match s.get::<Option<Vec<u16>>>("exposes.ports.tcp") {
                Ok(tcp) => assert!(tcp.is_some()),
                Err(e) => panic!(format!("error getting tcp: {:?}", e)),
            }

            s.try_into()
        };
        match f() {
            Ok(s) => {
                println!("Successful: {:?}", s);
                assert!(s.dependencies.is_some());
                assert!(s.dependencies.unwrap().platforms.get("windows").is_some())
            }
            Err(e) => panic!(format!("Failed to get configuration: {:?}", e)),
        }
    }
}
