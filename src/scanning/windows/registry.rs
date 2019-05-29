use winreg::enums::*;
use winreg::RegKey;

pub fn get_release_id() -> Option<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(cur_ver) = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion") {
        if let Ok(release_id) = cur_ver.get_value("ReleaseId") {
            return Some(release_id);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_release_id() {
        assert!(get_release_id().is_some(), "Failed to get release id");
    }
}
