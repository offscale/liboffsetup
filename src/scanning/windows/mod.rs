mod release_id;
mod winapi;

pub(crate) use self::release_id::get_release_id;
pub(crate) use self::winapi::get_platform_version;
