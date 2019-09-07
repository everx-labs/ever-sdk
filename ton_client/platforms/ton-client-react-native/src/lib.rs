// Android React Native

#[cfg(target_os="android")]
extern crate jni;
#[cfg(target_os="android")]
mod android;
#[cfg(target_os="android")]
pub use self::android::*;

// IOS React Native

#[cfg(target_os="ios")]
extern crate libc;
#[cfg(target_os="ios")]
mod ios;
#[cfg(target_os="ios")]
pub use self::ios::*;
