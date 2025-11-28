use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

lazy_static::lazy_static! {
    pub static ref LANGUAGE_LOADER: FluentLanguageLoader = {
        let loader = fluent_language_loader!();
        let requested_languages = DesktopLanguageRequester::requested_languages();
        i18n_embed::select(&loader, &Localizations, &requested_languages).unwrap();
        loader
    };
}

/// Get a localized string by key
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $message_id)
    }};
    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $message_id, $($args),*)
    }};
}

/// Initialize the localization system (called at startup)
pub fn init() {
    // This forces the lazy_static to initialize
    let _loader = &*LANGUAGE_LOADER;
}
