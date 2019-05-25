use core::borrow::Borrow;
use std::fs;

use itertools::Itertools;
use walkdir::{DirEntry, WalkDir};

/// PlatformScanner retrieves information based on what platform the binary is running on.
/// It is meant to be used for
/// 1) generating an initial config file
/// 2) validating the system matches what is expected when applying a configuration to it
pub struct PlatformScanner;

type PlatformBuild = String;
type Version = String;

impl PlatformScanner {
    /// search given directory for specific language dependencies ie LangDependencyName
    pub fn get_project_language_dependencies(dir: String) -> Option<Vec<LangDependencyName>> {
        let files: Vec<LangDependencyName> = WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
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

    fn _get_unix_platform_info() -> (PlatformName, Version) {
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
        (name, os.version)
    }

    fn _get_windows_platform_info() -> (PlatformName, Version, PlatformBuild) {
        (PlatformName::Windows, "0.0.0".into(), "0".into())
    }

    fn _get_architecture() -> Architecture {
        // Todo: get actual architecture
        return Architecture::x86_64;
    }

    pub fn get_current_platform() -> Platform {
        let mut p = Platform {
            arch: PlatformScanner::_get_architecture(),
            name: PlatformName::Unknown,
            version: "".into(),
            short_version: "".into(),
        };

        if cfg!(windows) {
            let (name, version, short_version) = PlatformScanner::_get_windows_platform_info();
            p.name = name;
            p.version = version;
            p.short_version = short_version;
        } else {
            let (name, version) = PlatformScanner::_get_unix_platform_info();
            p.name = name;
            p.version = version;
        }

        return p;
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

#[derive(Debug, PartialEq)]
pub enum Architecture {
    x86_32,
    x86_64,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub struct Platform {
    arch: Architecture,
    name: PlatformName,
    short_version: String,
    version: String,
}

#[cfg(test)]
mod tests {
    use crate::scanning::*;

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
        let p = PlatformScanner::get_current_platform();
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
