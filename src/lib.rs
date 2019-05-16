#[macro_use]
extern crate validator_derive;

use std::path::PathBuf;
use std::{
    collections::HashMap,
    env,
    string::{ParseError, ToString},
};

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

    debug: Option<bool>,
}

impl OffSetupCli {
    pub fn run() -> (OffSetupCli, OffSetup) {
        let args: OffSetupCli = OffSetupCli::from_args();
        let config = OffSetup::with_cli(args.clone());
        match config {
            Ok(c) => (args, c),
            Err(e) => panic!("Failed to load configuration file: {:#?}", e),
        }
    }
}

fn parse_string_list(input: &str) -> Result<Vec<String>, ParseError> {
    Ok(input.trim().split(',').map(ToString::to_string).collect())
}

#[derive(Clone, StructOpt, Debug, Deserialize)]
#[structopt(
    name = "offsetup",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
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

    /// Set install priority, override config specified if any
    #[structopt(
        short = "ip",
        long = "install-priority",
        parse(try_from_str = "parse_string_list"),
        help = "Comma separated list of priorities, will take precedence over whatever is in the the config file"
    )]
    install_priority: Option<Vec<String>>,

    #[structopt(
        short = "c",
        default_value = "offsetup.yml",
        raw(visible_aliases = r#"&["--config", "--configuration"]"#),
        help = "Specify configuration file"
    )]
    config_file: String,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Clone, StructOpt, Debug, Deserialize)]
enum Command {
    #[structopt(
        name = "new",
        raw(visible_aliases = r#"&["--new","init","--init"]"#),
        help = "Generate basic config file based on environment"
    )]
    Init,

    #[structopt(
        name = "install",
        raw(visible_aliases = r#"&["-i","--install"]"#),
        help = "Install the project, and all its dependencies"
    )]
    Install,

    #[structopt(
        name = "uninstall",
        raw(visible_aliases = r#"&["--uninstall","rm","--rm","remove","--remove"]"#),
        help = "Remove the project. Use --remove-shared to also remove the shared dependencies (eg: cmake)"
    )]
    Uninstall {
        #[structopt(long = "remove-shared")]
        remove_shared: bool,
    },

    // start, run, up
    #[structopt(
        name = "start",
        raw(visible_aliases = r#"&["--start","up","--up","run","--run"]"#),
        help = "Runs the project. Will inform user to run install [manually] if any of the dependencies aren't met"
    )]
    Start,

    // stop, down
    #[structopt(
        name = "stop",
        raw(visible_aliases = r#"&["--stop","down","--down"]"#),
        help = "Stops the project. Will have a nonzero exit code and a warning message if it's not started"
    )]
    Stop,
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
    fn with_cli(cli: OffSetupCli) -> Result<Self, ConfigError> {
        let mut config = Config::new();

        println!(
            "loading configuration from file: {:?}",
            cli.config_file.clone()
        );
        config.merge(File::from(PathBuf::from(cli.config_file)))?;

        println!("loading configuration from environment");
        config.merge(Environment::with_prefix("OFFSETUP"))?;

        if cli.install_priority.is_some() {
            let priorities = cli.install_priority.unwrap();
            println!("overriding install priorities to: {:?}", &priorities);

            if let Ok(Some(platforms)) =
                config.get::<Option<HashMap<String, Platform>>>("dependencies.platforms")
            {
                for name in platforms.keys() {
                    let path = format!("dependencies.platforms.{}.install_priority", name);

                    println!("setting {:?} to {:?}", path, &priorities);
                    config.set(&path, priorities.clone())?;
                }
            }
        }

        config.set("debug", Some(cli.debug))?;

        println!("configuration loaded");

        config.try_into()
    }
}

impl Default for OffSetup {
    fn default() -> Self {
        const DEFAULT: fn() -> Result<OffSetup, ConfigError> = || {
            let mut config = Config::new();

            // Start off by merging in the "default" configuration file
            config.merge(File::from(PathBuf::from("offsetup.yml")))?;

            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
            config.merge(File::from(PathBuf::from("config").join(run_mode)).required(false))?;

            // Add in settings from the environment (with a prefix of OFFSETUP)
            // Eg.. `OFFSETUP_DEBUG=1 ./target/app` would set the `debug` key
            config.merge(Environment::with_prefix("OFFSETUP"))?;

            // Now that we're done, let's access our configuration
            println!("debug: {:?}", config.get_bool("debug"));

            // You can deserialize (and thus freeze) the entire configuration
            config.try_into()
        };
        DEFAULT().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_use_cli() {
        let (args, config) = OffSetupCli::run();
        println!("args: {:?}", args);
        println!("config: {:#?}", config);
    }

    #[test]
    fn can_read_simple_ports() {
        println!("testing ports...");
        let get_exposes = || -> Result<Exposes, ConfigError> {
            let mut config = Config::default();
            config.merge(File::from(PathBuf::from("examples").join("exposes")))?;
            println!("merged: {:#?}", config);

            match config.get::<Option<Vec<u16>>>("ports.tcp") {
                Ok(udp) => assert!(udp.is_some()),
                Err(e) => panic!(format!("error getting tdp: {:?}", e)),
            }

            config.try_into()
        };
        match get_exposes() {
            Ok(exposes) => println!("Successful: {:#?}", exposes),
            Err(e) => panic!(format!("Failed to get configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_read_simple_file() {
        let get_offsetup = || -> Result<OffSetup, ConfigError> {
            let mut config = Config::default();
            config.merge(File::from(PathBuf::from("examples").join("simple")))?;
            println!("merged: {:#?}", config);

            match config.get::<Option<Vec<u16>>>("exposes.ports.tcp") {
                Ok(tcp) => assert!(tcp.is_some()),
                Err(e) => panic!(format!("error getting tcp: {:?}", e)),
            }

            config.try_into()
        };
        match get_offsetup() {
            Ok(offsetup) => {
                println!("Successful simple: {:#?}", offsetup);
                assert_eq!(offsetup.name, "random python project name")
            }
            Err(e) => panic!(format!("Failed to get simple configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_read_source_file() {
        let get_source = || -> Result<Source, ConfigError> {
            let mut config = Config::default();
            config.merge(File::from(PathBuf::from("examples").join("source")))?;
            println!("merged: {:#?}", config);

            match config.get::<Option<Download>>("download") {
                Ok(download) => assert!(download.is_some()),
                Err(e) => panic!(format!("error getting download from Source file: {:?}", e)),
            }

            config.try_into()
        };
        match get_source() {
            Ok(source) => {
                println!("Successful Source: {:#?}", source);
                assert_eq!(
                    source.download.unwrap().uri.hostname.unwrap().to_string(),
                    "download.redis.io"
                )
            }
            Err(e) => panic!(format!("Failed to get Source configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_validate_valid_source() {
        let get_source = || -> Result<Source, ConfigError> {
            let mut config = Config::default();
            config.merge(File::from(
                PathBuf::from("examples").join("valid_source_download"),
            ))?;
            println!("merged: {:#?}", config);

            config.try_into()
        };
        match get_source() {
            Ok(source) => {
                println!("Successfully loaded valid Source: {:#?}", source);
                assert_eq!(
                    source
                        .clone()
                        .download
                        .unwrap()
                        .uri
                        .hostname
                        .unwrap()
                        .to_string(),
                    "download.redis.io"
                );
                match source.validate() {
                    Ok(_) => (),
                    Err(e) => panic!(format!("Valid Source download failed validation: {:?}", e)),
                }
            }
            Err(e) => panic!(format!("Failed to get valid Source configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_validate_invalid_source_no1() {
        let get_source = || -> Result<Source, ConfigError> {
            let mut config = Config::default();
            config.merge(File::from(
                PathBuf::from("examples").join("invalid_source_download_no1"),
            ))?;
            println!("merged: {:#?}", config);

            config.try_into()
        };
        match get_source() {
            Ok(source) => {
                println!("Successfully loaded invalid Source no1: {:#?}", source);
                assert_eq!(
                    source
                        .clone()
                        .download
                        .unwrap()
                        .uri
                        .hostname
                        .unwrap()
                        .to_string(),
                    "download.redis.io"
                );
                assert_eq!(true, source.clone().download_directory.is_none());
                match source.validate() {
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
        let get_source = || -> Result<Source, ConfigError> {
            let mut config = Config::default();
            config.merge(File::from(
                PathBuf::from("examples").join("invalid_source_download_no2"),
            ))?;
            println!("merged: {:#?}", config);

            config.try_into()
        };
        match get_source() {
            Ok(source) => {
                println!("Successfully loaded invalid Source 2: {:#?}", source);
                assert_eq!(
                    source
                        .clone()
                        .download
                        .unwrap()
                        .uri
                        .hostname
                        .unwrap()
                        .to_string(),
                    "download.redis.io"
                );
                assert_eq!(true, source.clone().download_directory.is_none());
                match source.validate() {
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
        let get_system = || -> Result<System, ConfigError> {
            let mut config = Config::default();
            config.merge(File::from(PathBuf::from("examples").join("system")))?;
            println!("merged: {:#?}", config);

            match config.get::<Option<Vec<String>>>("apt") {
                Ok(tcp) => assert!(tcp.is_some()),
                Err(e) => panic!(format!("error getting apt from system file: {:?}", e)),
            }

            config.try_into()
        };
        match get_system() {
            Ok(system) => println!("Successful system: {:#?}", system),
            Err(e) => panic!(format!("Failed to get system configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_read_platform_file() {
        let get_platform = || -> Result<Platform, ConfigError> {
            let mut config = Config::default();
            config.merge(File::from(PathBuf::from("examples").join("platform")))?;
            println!("merged: {:#?}", config);

            let key = "system.apt";
            match config.get::<Option<Vec<String>>>(key) {
                Ok(apt) => {
                    println!("{:?}: {:?}", key, apt);
                    assert!(apt.is_some())
                }
                Err(e) => panic!(format!("error getting apt from platform file: {:?}", e)),
            }

            config.try_into()
        };
        match get_platform() {
            Ok(platform) => println!("Successful platform: {:#?}", platform),
            Err(e) => panic!(format!("Failed to get platform configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_read_dependencies_file() {
        let get_dependencies = || -> Result<Dependencies, ConfigError> {
            let mut config = Config::default();
            config.merge(File::from(PathBuf::from("examples").join("dependencies")))?;
            println!("merged: {:#?}", config);

            const KEY: &'static str = "platforms.ubuntu.system.apt";
            match config.get::<Option<Vec<String>>>(KEY) {
                Ok(apt) => {
                    println!("{:?}: {:?}", KEY, apt);
                    assert!(apt.is_some())
                }
                Err(e) => panic!(format!("error getting apt from dependencies: {:?}", e)),
            }

            config.try_into()
        };
        match get_dependencies() {
            Ok(dependencies) => println!("Successful dependencies: {:#?}", dependencies),
            Err(e) => panic!(format!("Failed to get dependencies configuration: {:?}", e)),
        }
    }

    #[test]
    fn can_read_redis_file() {
        let mut config = Config::default();
        config
            .merge(File::from(PathBuf::from("examples").join("redis")))
            .unwrap();
        println!("merged: {:#?}", config);

        println!(
            "redis platforms: {:#?}",
            config.get::<Option<HashMap<String, Platform>>>("dependencies.platforms")
        );

        match config.get::<Platform>("dependencies.platforms.windows") {
            Ok(windows) => {
                assert!(windows.arch.is_some());
                assert_eq!(windows.arch.unwrap(), "x86_64")
            }
            Err(e) => panic!(format!("error getting windows platform: {:?}", e)),
        }

        match config.get::<Option<Vec<u16>>>("exposes.ports.tcp") {
            Ok(tcp) => assert!(tcp.is_some()),
            Err(e) => panic!(format!("error getting tcp: {:?}", e)),
        }

        match config.try_into() as Result<OffSetup, ConfigError> {
            Ok(offsetup) => {
                println!("Successful redis: {:#?}", offsetup);
                assert!(offsetup.dependencies.is_some());
                assert!(offsetup
                    .dependencies
                    .clone()
                    .unwrap()
                    .platforms
                    .unwrap()
                    .get("windows")
                    .is_some());
                assert!(offsetup
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
