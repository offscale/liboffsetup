mod registry;
mod winapi;

pub(crate) use self::registry::get_release_id;
pub(crate) use self::winapi::get_platform_version;
