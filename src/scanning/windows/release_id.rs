use std::collections::HashMap;

use lazy_static::lazy_static;

lazy_static! {
    /// build number to release id map, eg 10.0.*9926* -> 1507
    static ref WINDOWS_10_RELASE_MAP: HashMap<(u64,u64), &'static str> = {
        let m: HashMap<(u64,u64), &'static str> = [
            ((9841, 10240),  "1507"),
            ((10525, 10586), "1511"),
            ((11082, 14393), "1607"),
            ((14901, 15063), "1703"),
            ((16170, 16299), "1709"),
            ((16353, 17134), "1803"),
            ((17604, 17763), "1809"),
            ((18204, 18362), "1903"),
            ((18836, 18908), "20H1"), // current Windows 10 preview
        ].iter().cloned().collect();
        m
    };
}

/// Convert given build number to release id
pub fn get_release_id(build: &u64) -> Option<String> {
    for ((preview_start, release_build), release_id) in WINDOWS_10_RELASE_MAP.iter() {
        if (preview_start..=release_build).contains(&build) {
            return Some(release_id.to_string());
        };
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_release_id() {
        assert_eq!(
            "1507",
            get_release_id(&9841).unwrap(),
            "Failed to get current preview release id"
        );
        assert_eq!(
            "1507",
            get_release_id(&10240).unwrap(),
            "Failed to get correct release id"
        );

        assert_eq!(
            "1809",
            get_release_id(&17604).unwrap(),
            "Failed to get current preview release id"
        );
        assert_eq!(
            "1809",
            get_release_id(&17763).unwrap(),
            "Failed to get correct release id"
        );

        assert_eq!(
            "1903",
            get_release_id(&18204).unwrap(),
            "Failed to get current preview release id"
        );
        assert_eq!(
            "1903",
            get_release_id(&18300).unwrap(),
            "Failed to get current preview release id"
        );
        assert_eq!(
            "1903",
            get_release_id(&18362).unwrap(),
            "Failed to get correct release id"
        );
    }
}
