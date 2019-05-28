use winreg::enums::*;
use winreg::RegKey;

pub fn get_release_id() -> Some(String) {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(cur_ver) = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion") {
        if let Ok(release_id) = cur_ver.get_value("ReleaseId") {
            Some(release_id)
        }
    }
    None
}
