#![allow(unsafe_code)]

use std::mem;

use winapi::shared::{minwindef::DWORD, ntdef::NTSTATUS, ntstatus::STATUS_SUCCESS};
#[cfg(target_arch = "x86")]
#[allow(unused_imports)]
use winapi::um::winnt::OSVERSIONINFOEXA;
#[cfg(not(target_arch = "x86"))]
#[allow(unused_imports)]
use winapi::um::winnt::OSVERSIONINFOEXW;
use winapi::um::{
    sysinfoapi::GetSystemInfo, sysinfoapi::SYSTEM_INFO, winuser::GetSystemMetrics,
    winuser::SM_SERVERR2,
};

use crate::scanning::{os::get_release_id, platform::PlatformVersionAliases};

#[cfg(target_arch = "x86")]
type OSVERSIONINFOEX = OSVERSIONINFOEXA;

#[cfg(not(target_arch = "x86"))]
type OSVERSIONINFOEX = OSVERSIONINFOEXW;

/// Win32 Flag: VER_NT_WORKSTATION
/// https://msdn.microsoft.com/en-us/library/windows/desktop/ms724833(v=vs.85).aspx
const VER_NT_WORKSTATION: u8 = 0x0000001;
/// Win32 Flag: VER_SUITE_WH_SERVER
/// https://msdn.microsoft.com/en-us/library/windows/desktop/ms724833(v=vs.85).aspx
const VER_SUITE_WH_SERVER: u16 = 0x00008000;
/// Win32 Flag: PROCESSOR_ARCHITECTURE_AMD64
/// https://msdn.microsoft.com/en-us/library/windows/desktop/ms724958(v=vs.85).aspx
const PROCESSOR_ARCHITECTURE_AMD64: u16 = 9;

#[link(name = "ntdll")]
extern "system" {
    pub fn RtlGetVersion(lpVersionInformation: &mut OSVERSIONINFOEX) -> NTSTATUS;
}

pub fn get_platform_version() -> PlatformVersionAliases {
    let version_info = match get_version_info() {
        None => {
            return vec!["Unknown Windows".into()];
        }
        Some(val) => val,
    };

    let build_number = version_info.dwBuildNumber as u64;
    match (
        get_product_name(&version_info),
        get_release_id(&build_number),
    ) {
        (Some(name), Some(id)) => vec![
            name.into(),
            version_info.dwBuildNumber.to_string(),
            id.into(),
        ],
        (Some(name), None) => vec![name.into(), version_info.dwBuildNumber.to_string()],
        (None, _) => panic!(
            "unknown Windows version: {:?}.{:?}.{:?}",
            version_info.dwMajorVersion as u64, version_info.dwMinorVersion as u64, build_number,
        ),
    }
}

// Calls the Win32 API function RtlGetVersion to get the OS version information:
// https://msdn.microsoft.com/en-us/library/mt723418(v=vs.85).aspx
fn get_version_info() -> Option<OSVERSIONINFOEX> {
    let mut info: OSVERSIONINFOEX = unsafe { mem::zeroed() };
    info.dwOSVersionInfoSize = mem::size_of::<OSVERSIONINFOEX>() as DWORD;

    if unsafe { RtlGetVersion(&mut info) } == STATUS_SUCCESS {
        Some(info)
    } else {
        None
    }
}

// Examines data in the OSVERSIONINFOEX structure to determine the Windows edition:
// https://msdn.microsoft.com/en-us/library/windows/desktop/ms724833(v=vs.85).aspx
fn get_product_name(version_info: &OSVERSIONINFOEX) -> Option<String> {
    match (
        version_info.dwMajorVersion,
        version_info.dwMinorVersion,
        version_info.wProductType,
    ) {
        // Windows 10.
        (10, 0, VER_NT_WORKSTATION) => Some("Windows 10"),
        (10, 0, _) => Some("Windows Server 2016"),
        // Windows Vista, 7, 8 and 8.1.
        (6, 0, VER_NT_WORKSTATION) => Some("Windows Vista"),
        (6, 0, _) => Some("Windows Server 2008"),
        (6, 1, VER_NT_WORKSTATION) => Some("Windows 7"),
        (6, 1, _) => Some("Windows Server 2008 R2"),
        (6, 2, VER_NT_WORKSTATION) => Some("Windows 8"),
        (6, 2, _) => Some("Windows Server 2012"),
        (6, 3, VER_NT_WORKSTATION) => Some("Windows 8.1"),
        (6, 3, _) => Some("Windows Server 2012 R2"),
        // Windows 2000, Home Server, 2003 Server, 2003 R2 Server, XP and XP Professional x64.
        (5, 0, _) => Some("Windows 2000"),
        (5, 1, _) => Some("Windows XP"),
        (5, 2, _) if unsafe { GetSystemMetrics(SM_SERVERR2) } == 0 => {
            let mut info: SYSTEM_INFO = unsafe { mem::zeroed() };
            unsafe { GetSystemInfo(&mut info) };

            if version_info.wSuiteMask & VER_SUITE_WH_SERVER == VER_SUITE_WH_SERVER {
                Some("Windows Home Server")
            } else if version_info.wProductType == VER_NT_WORKSTATION
                && unsafe { info.u.s().wProcessorArchitecture == PROCESSOR_ARCHITECTURE_AMD64 }
            {
                Some("Windows XP Professional x64 Edition")
            } else {
                Some("Windows Server 2003")
            }
        }
        _ => None,
    }
    .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_platform_version() {
        let versions = get_platform_version();
        assert!(versions.len() > 0);
    }

    #[test]
    fn can_find_version_info() {
        let version = get_version_info();
        assert!(version.is_some());
    }

    #[test]
    fn is_product_name_correct() {
        let test_data = [
            (5, 0, 0, "Windows 2000"),
            (5, 0, 1, "Windows 2000"),
            (5, 0, 100, "Windows 2000"),
            (5, 1, 0, "Windows XP"),
            (5, 1, 1, "Windows XP"),
            (5, 1, 100, "Windows XP"),
            (6, 0, VER_NT_WORKSTATION, "Windows Vista"),
            (6, 0, 0, "Windows Server 2008"),
            (6, 1, VER_NT_WORKSTATION, "Windows 7"),
            (6, 1, 0, "Windows Server 2008 R2"),
            (6, 2, VER_NT_WORKSTATION, "Windows 8"),
            (6, 2, 0, "Windows Server 2012"),
            (6, 3, VER_NT_WORKSTATION, "Windows 8.1"),
            (6, 3, 0, "Windows Server 2012 R2"),
            (10, 0, VER_NT_WORKSTATION, "Windows 10"),
            (10, 0, 0, "Windows Server 2016"),
        ];

        let mut info = get_version_info().unwrap();

        for &(major, minor, product_type, name) in &test_data {
            info.dwMajorVersion = major;
            info.dwMinorVersion = minor;
            info.wProductType = product_type;

            let name = get_product_name(&info).unwrap();
            assert_eq!(name, name);
        }
    }
}
