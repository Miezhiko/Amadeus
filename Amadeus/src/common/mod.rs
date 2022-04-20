#[macro_use] pub mod paths;
pub mod constants;
pub mod aka;
pub mod msg;
pub mod log;
pub mod options;
pub mod i18n;
pub mod colors;
pub mod help;
pub mod db;
pub mod system;
pub mod voice;

#[cfg(not(target_os = "windows"))]
pub mod salieri;

#[cfg(feature = "voice_analysis")]
pub mod voice_to_text;
#[cfg(feature = "voice_analysis")]
pub mod voice_analysis;
