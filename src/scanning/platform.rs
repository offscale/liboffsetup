use core::borrow::Borrow;

use itertools::Itertools;
use walkdir::WalkDir;

use crate::scanning::os;
use std::str::FromStr;

/// PlatformScanner retrieves information based on what platform the binary is running on.
/// It is meant to be used for
/// 1) generating an initial config file
/// 2) validating the system matches what is expected when applying a configuration to it
pub struct PlatformScanner;

pub type PlatformVersionAliases = Vec<String>;

#[cfg(target_arch = "x86")]
fn get_architecture() -> Architecture {
    Architecture::X86_32
}

#[cfg(target_arch = "x86_64")]
fn get_architecture() -> Architecture {
    Architecture::X86_64
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
fn get_architecture() -> Architecture {
    Architecture::Unknown
}

impl PlatformScanner {
    /// search given directory for specific language dependencies ie LangDependencyName
    pub fn get_project_language_dependencies(dir: String) -> Option<Vec<LangDependencyName>> {
        let files: Vec<LangDependencyName> = WalkDir::new(dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter_map(|d| {
                if let Some(ext) = d.path().extension() {
                    match ext
                        .to_str()
                        .expect("failed to convert to string")
                        .to_lowercase()
                        .borrow()
                    {
                        "go" => Some(LangDependencyName::Go),
                        "rs" => Some(LangDependencyName::Rust),
                        "js" | "ts" => Some(LangDependencyName::NodeJS),
                        "py" => Some(LangDependencyName::Python),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .unique()
            .collect();
        if files.is_empty() {
            None
        } else {
            Some(files)
        }
    }

    fn _get_unix_platform_info() -> (PlatformName, PlatformVersionAliases) {
        let os = os_type::current_platform();
        let name = match os.os_type {
            os_type::OSType::Arch => PlatformName::Arch,
            os_type::OSType::CentOS => PlatformName::CentOS,
            os_type::OSType::Debian => PlatformName::Debian,
            os_type::OSType::OSX => PlatformName::MacOSX,
            os_type::OSType::Manjaro => PlatformName::Manjaro,
            os_type::OSType::Redhat => PlatformName::Redhat,
            os_type::OSType::Ubuntu => PlatformName::Ubuntu,
            os_type::OSType::Unknown | _ => PlatformName::Unknown,
        };
        (name, vec![os.version])
    }

    fn _get_windows_platform_info() -> (PlatformName, PlatformVersionAliases) {
        let versions = os::get_platform_version();
        (PlatformName::Windows, versions)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum LangDependencyName {
    Go,
    NodeJS,
    Python,
    Rust,
}

#[derive(Debug, PartialEq)]
pub struct LangDependency {
    name: LangDependencyName,
    version: String,
}

#[derive(Debug, PartialEq)]
pub enum PlatformName {
    Arch,
    CentOS,
    Debian,
    MacOSX,
    Manjaro,
    Redhat,
    Ubuntu,
    Unknown,
    Windows,
}

#[derive(Debug)]
pub enum PlatformNameParsingError {
    InvalidPlatform,
}

impl FromStr for PlatformName {
    type Err = PlatformNameParsingError;
    fn from_str(name: &str) -> Result<PlatformName, PlatformNameParsingError> {
        match name {
            "arch" => Ok(PlatformName::Arch),
            "centos" => Ok(PlatformName::CentOS),
            "debian" => Ok(PlatformName::Debian),
            "macos" => Ok(PlatformName::MacOSX),
            "manjaro" => Ok(PlatformName::Manjaro),
            "redhat" => Ok(PlatformName::Redhat),
            "ubuntu" => Ok(PlatformName::Ubuntu),
            "unknown" => Ok(PlatformName::Unknown),
            "windows" => Ok(PlatformName::Windows),
            _ => Err(PlatformNameParsingError::InvalidPlatform),
        }
    }
}

impl ToString for PlatformName {
    fn to_string(&self) -> String {
        match self {
            PlatformName::Arch => "arch",
            PlatformName::CentOS => "centos",
            PlatformName::Debian => "debian",
            PlatformName::MacOSX => "macos",
            PlatformName::Manjaro => "manjaro",
            PlatformName::Redhat => "redhat",
            PlatformName::Ubuntu => "ubuntu",
            PlatformName::Unknown => "unknown",
            PlatformName::Windows => "windows",
        }
        .to_string()
    }
}

#[derive(Debug, PartialEq)]
pub enum Architecture {
    X86_32,
    X86_64,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub struct Platform {
    pub arch: Architecture,
    pub name: PlatformName,
    versions: PlatformVersionAliases,
}

impl Default for Platform {
    fn default() -> Platform {
        let mut p = Platform {
            arch: get_architecture(),
            name: PlatformName::Unknown,
            versions: vec![],
        };

        if cfg!(windows) {
            let (name, version) = PlatformScanner::_get_windows_platform_info();
            p.name = name;
            p.versions = version;
        } else {
            let (name, version) = PlatformScanner::_get_unix_platform_info();
            p.name = name;
            p.versions = version;
        }

        p
    }
}

#[cfg(test)]
mod tests {
    use crate::scanning::platform::*;

    #[test]
    fn list_go_dependencies() {
        let result =
            PlatformScanner::get_project_language_dependencies("examples/scanner/new/go".into());

        assert!(result.is_some(), "Could not find dependency");
        let deps = result.unwrap();
        assert_eq!(1, deps.len(), "failed to find one dependency");
        assert_eq!(LangDependencyName::Go, deps[0])
    }

    #[test]
    fn list_mixed_go_rust_dependencies() {
        let result = PlatformScanner::get_project_language_dependencies(
            "examples/scanner/new/mixed_go_rust".into(),
        );

        assert!(result.is_some(), "Could not find dependencies");
        let deps = result.unwrap();
        assert_eq!(2, deps.len(), "failed to find both dependencies");

        let first = &deps[0];
        assert!(*first == LangDependencyName::Go || *first == LangDependencyName::Rust);
        assert!(*first != LangDependencyName::NodeJS && *first != LangDependencyName::Python);

        let second = &deps[1];
        assert!(*second == LangDependencyName::Go || *second == LangDependencyName::Rust);
        assert!(*second != LangDependencyName::NodeJS && *second != LangDependencyName::Python);
    }

    #[test]
    fn list_nodejs_dependencies() {
        let result = PlatformScanner::get_project_language_dependencies(
            "examples/scanner/new/nodejs".into(),
        );

        assert!(result.is_some(), "Could not find dependency");
        let deps = result.unwrap();
        assert_eq!(1, deps.len(), "failed to find one dependency");

        let first = &deps[0];
        assert_eq!(LangDependencyName::NodeJS, *first);
        assert_ne!(LangDependencyName::Python, *first);
        assert_ne!(LangDependencyName::Go, *first);
        assert_ne!(LangDependencyName::Rust, *first);
    }

    #[test]
    fn list_python_dependencies() {
        let result = PlatformScanner::get_project_language_dependencies(
            "examples/scanner/new/python".into(),
        );

        assert!(result.is_some(), "Could not find dependency");
        let deps = result.unwrap();
        assert_eq!(1, deps.len(), "failed to find one dependency");
        assert_eq!(LangDependencyName::Python, deps[0])
    }

    #[test]
    fn list_rust_dependencies() {
        let result =
            PlatformScanner::get_project_language_dependencies("examples/scanner/new/rust".into());

        assert!(result.is_some(), "Could not find dependency");
        let deps = result.unwrap();
        assert_eq!(1, deps.len(), "failed to find one dependency");
        assert_eq!(LangDependencyName::Rust, deps[0])
    }

    #[test]
    fn can_get_platform() {
        let p = Platform::default();
        if cfg!(windows) {
            assert_eq!(p.name, PlatformName::Windows, "should be Windows")
        } else if cfg!(unix) {
            match p.name {
                PlatformName::Arch => println!("Found Arch platform"),
                PlatformName::CentOS => println!("Found CentOS platform"),
                PlatformName::Debian => println!("Found Debian platform"),
                PlatformName::MacOSX => println!("Found Mac OSX platform"),
                PlatformName::Manjaro => println!("Found Manjaro platform"),
                PlatformName::Redhat => println!("Found Redhat platform"),
                PlatformName::Ubuntu => println!("Found Ubuntu platform"),
                PlatformName::Unknown | _ => panic!("Found unsupported unix platform: {:?}", p),
            }
        }
        assert_ne!(
            p.arch,
            Architecture::Unknown,
            "should know the architecture"
        )
    }
}
