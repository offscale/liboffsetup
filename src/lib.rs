#[macro_use]
extern crate validator_derive;

use std::{collections::HashMap, env};

use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Deserializer};
use structopt::StructOpt;
use urlparse::{urlparse, Url};
use validator::{Validate, ValidationError};

// Since structopt/clap does not support config file, only cli and env, we split the two between
// 1) config for file and environment
// 2) structopt for CLI
#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
struct System {
    /// Linux
    // https://manpages.debian.org/stretch/apt/apt.8.en.html
    apt: Option<Vec<String>>,
    // https://manpages.debian.org/stretch/apt/apt-get.8.en.html
    apt_get: Option<Vec<String>>,
    // https://manpages.debian.org/stretch/aptitude/aptitude.8.en.html
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
    // https://metacpan.org/pod/distribution/urpmi/pod/8/urpmihowto.pod
    urpmi: Option<Vec<String>>,
    // https://slackpkg.org/documentation.html
    slackpkg: Option<Vec<String>>,
    // https://software.jaos.org/git/slapt-get/plain/README
    slapt_get: Option<Vec<String>>,
    // https://docs.snapcraft.io/getting-started
    snap: Option<Vec<String>>,
    // http://www.brunolinux.com/03-Installing_Software/Swaret.html
    swaret: Option<Vec<String>>,

    /// Windows
    // https://chocolatey.org
    choco: Option<Vec<String>>,

    /// OS X
    // https://brew.sh
    brew: Option<Vec<String>>,

    /// BSD
    // https://www.freebsd.org/cgi/man.cgi?query=pkg
    pkg: Option<Vec<String>>,

    /// Windows, Linux, OS X
    // https://0install.de/docs/commands/
    _0install: Option<Vec<String>>,

    apk: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize)]
struct Dependencies {
    applications: Option<HashMap<String, Application>>,
    platforms: Option<HashMap<String, Platform>>,
}

#[derive(Clone, Debug, Deserialize)]
struct Application {
    pkg: Option<String>,
    version: Option<String>,
    env: Option<String>,

    install_priority: Option<Vec<String>>,
    skip_install: Option<bool>,
    fail_silently: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
struct Platform {
    versions: Vec<String>,

    arch: Option<String>,

    source: Option<Source>,

    system: Option<System>,
    pre_install: Option<Vec<String>>,
    install_priority: Option<Vec<String>>,
    skip_install: Option<bool>,
    fail_silently: Option<bool>,
}

fn validate_source_download(data: &Source) -> Result<(), ValidationError> {
    if data.download_directory.is_none() && data.download.is_some() {
        return Err(ValidationError::new("download_directory_required"));
    }

    if data.download_directory.is_some() && data.download.is_none() {
        return Err(ValidationError::new("download_is_required"));
    }

    Ok(())
}

#[derive(Clone, Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_source_download", skip_on_field_errors = "false"))]
struct Source {
    // TODO: find out if automatic/implicit validate() call can be made after Deserialize
    download_directory: Option<String>,
    download: Option<Download>,

    system: Option<System>,
}

pub trait DeserializeWith: Sized {
    fn deserialize_with<'de, D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

impl DeserializeWith for Url {
    fn deserialize_with<'de, D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(de)?;
        Ok(urlparse(&s))
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Download {
    extract: Option<bool>,
    sha512: String,
    shareable: Option<bool>,
    #[serde(deserialize_with = "Url::deserialize_with")]
    uri: Url,
}

#[derive(Clone, Debug, Deserialize)]
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
        let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
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
            println!("merged: {:#?}", s);

            match s.get::<Option<Vec<u16>>>("ports.tcp") {
                Ok(udp) => assert!(udp.is_some()),
                Err(e) => panic!(format!("error getting tdp: {:?}", e)),
            }

            s.try_into()
        };
        match f() {
            Ok(s) => println!("Successful: {:#?}", s),
            Err(e) => panic!(format!("Failed to get configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_read_simple_file() {
        let f = || -> Result<OffSetup, ConfigError> {
            let mut s = Config::new();

            s.merge(File::with_name("examples/simple"))?;
            println!("merged: {:#?}", s);

            match s.get::<Option<Vec<u16>>>("exposes.ports.tcp") {
                Ok(tcp) => assert!(tcp.is_some()),
                Err(e) => panic!(format!("error getting tcp: {:?}", e)),
            }

            s.try_into()
        };
        match f() {
            Ok(s) => {
                println!("Successful simple: {:#?}", s);
                assert_eq!(s.name, "random python project name")
            }
            Err(e) => panic!(format!("Failed to get simple configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_read_source_file() {
        let f = || -> Result<Source, ConfigError> {
            let mut s = Config::new();

            s.merge(File::with_name("examples/source"))?;
            println!("merged: {:#?}", s);

            match s.get::<Option<Download>>("download") {
                Ok(download) => assert!(download.is_some()),
                Err(e) => panic!(format!("error getting download from Source file: {:?}", e)),
            }

            s.try_into()
        };
        match f() {
            Ok(s) => {
                println!("Successful Source: {:#?}", s);
                assert_eq!(
                    s.download.unwrap().uri.hostname.unwrap().to_string(),
                    "download.redis.io"
                )
            }
            Err(e) => panic!(format!("Failed to get Source configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_validate_valid_source() {
        let f = || -> Result<Source, ConfigError> {
            let mut s = Config::new();

            s.merge(File::with_name("examples/valid_source_download"))?;
            println!("merged: {:#?}", s);

            s.try_into()
        };
        match f() {
            Ok(s) => {
                println!("Successfully loaded valid Source: {:#?}", s);
                assert_eq!(
                    s.clone()
                        .download
                        .unwrap()
                        .uri
                        .hostname
                        .unwrap()
                        .to_string(),
                    "download.redis.io"
                );
                match s.validate() {
                    Ok(_) => (),
                    Err(e) => panic!(format!("Valid Source download failed validation: {:?}", e)),
                }
            }
            Err(e) => panic!(format!("Failed to get valid Source configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_validate_invalid_source_no1() {
        let f = || -> Result<Source, ConfigError> {
            let mut s = Config::new();

            s.merge(File::with_name("examples/invalid_source_download_no1"))?;
            println!("merged: {:#?}", s);

            s.try_into()
        };
        match f() {
            Ok(s) => {
                println!("Successfully loaded invalid Source no1: {:#?}", s);
                assert_eq!(
                    s.clone()
                        .download
                        .unwrap()
                        .uri
                        .hostname
                        .unwrap()
                        .to_string(),
                    "download.redis.io"
                );
                assert_eq!(true, s.clone().download_directory.is_none());
                match s.validate() {
                    Ok(valid) => panic!(format!(
                        "Invalid Source download is not supposed to pass: {:#?}",
                        valid
                    )),
                    Err(_) => (),
                }
            }
            Err(e) => panic!(format!(
                "Failed to get invalid Source configuration: {:?}",
                e
            )),
        }
    }

    #[test]
    fn can_validate_invalid_source_no2() {
        let f = || -> Result<Source, ConfigError> {
            let mut s = Config::new();

            s.merge(File::with_name("examples/invalid_source_download_no2"))?;
            println!("merged: {:#?}", s);

            s.try_into()
        };
        match f() {
            Ok(s) => {
                println!("Successfully loaded invalid Source 2: {:#?}", s);
                assert_eq!(
                    s.clone()
                        .download
                        .unwrap()
                        .uri
                        .hostname
                        .unwrap()
                        .to_string(),
                    "download.redis.io"
                );
                assert_eq!(true, s.clone().download_directory.is_none());
                match s.validate() {
                    Ok(valid) => panic!(format!(
                        "Invalid Source download 2 is not supposed to pass: {:#?}",
                        valid
                    )),
                    Err(_) => (),
                }
            }
            Err(e) => panic!(format!(
                "Failed to get invalid Source 2 configuration: {:?}",
                e
            )),
        }
    }

    #[test]
    fn can_read_system_file() {
        let f = || -> Result<System, ConfigError> {
            let mut s = Config::new();

            s.merge(File::with_name("examples/system"))?;
            println!("merged: {:#?}", s);

            match s.get::<Option<Vec<String>>>("apt") {
                Ok(tcp) => assert!(tcp.is_some()),
                Err(e) => panic!(format!("error getting apt from system file: {:?}", e)),
            }

            s.try_into()
        };
        match f() {
            Ok(s) => println!("Successful system: {:#?}", s),
            Err(e) => panic!(format!("Failed to get system configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_read_platform_file() {
        let f = || -> Result<Platform, ConfigError> {
            let mut s = Config::new();

            s.merge(File::with_name("examples/platform"))?;
            println!("merged: {:#?}", s);

            let key = "system.apt";
            match s.get::<Option<Vec<String>>>(key) {
                Ok(apt) => {
                    println!("{:?}: {:?}", key, apt);
                    assert!(apt.is_some())
                }
                Err(e) => panic!(format!("error getting apt from platform file: {:?}", e)),
            }

            s.try_into()
        };
        match f() {
            Ok(s) => println!("Successful platform: {:#?}", s),
            Err(e) => panic!(format!("Failed to get platform configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_read_dependencies_file() {
        let f = || -> Result<Dependencies, ConfigError> {
            let mut s = Config::new();

            s.merge(File::with_name("examples/dependencies"))?;
            println!("merged: {:#?}", s);

            let key = "platforms.ubuntu.system.apt";
            match s.get::<Option<Vec<String>>>(key) {
                Ok(apt) => {
                    println!("{:?}: {:?}", key, apt);
                    assert!(apt.is_some())
                }
                Err(e) => panic!(format!("error getting apt from dependencies: {:?}", e)),
            }

            s.try_into()
        };
        match f() {
            Ok(s) => println!("Successful dependencies: {:#?}", s),
            Err(e) => panic!(format!("Failed to get dependencies configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_read_redis_file() {
        let f = || -> Result<OffSetup, ConfigError> {
            let mut s = Config::new();

            s.merge(File::with_name("examples/redis"))?;
            println!("merged: {:#?}", s);

            println!(
                "redis platforms: {:#?}",
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
                println!("Successful redis: {:#?}", s);
                assert!(s.dependencies.is_some());
                assert!(s
                    .dependencies
                    .clone()
                    .unwrap()
                    .platforms
                    .unwrap()
                    .get("windows")
                    .is_some());
                assert!(s
                    .dependencies
                    .unwrap()
                    .platforms
                    .unwrap()
                    .get("ubuntu")
                    .unwrap()
                    .system
                    .clone()
                    .unwrap()
                    .apt
                    .is_some());
            }
            Err(e) => panic!(format!("Failed to get redis configuration: {:?}", e)),
        }
    }
}
