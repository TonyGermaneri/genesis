//! Localization System
//!
//! Multi-language support with fallback, interpolation, and plural forms.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during localization operations
#[derive(Debug, Error)]
pub enum LocaleError {
    /// I/O error reading locale file
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error parsing locale file
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Locale not found
    #[error("Locale not found: {0}")]
    LocaleNotFound(String),

    /// Key not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

/// A locale file containing translations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleFile {
    /// Locale code (e.g., "en", "es", "ja")
    pub locale: String,
    /// Human-readable locale name
    pub name: String,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
    /// Translation strings: key -> value
    pub strings: HashMap<String, String>,
    /// Plural forms: key -> (singular, plural)
    #[serde(default)]
    pub plurals: HashMap<String, PluralForms>,
}

/// Plural form variations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluralForms {
    /// Zero items (optional)
    #[serde(default)]
    pub zero: Option<String>,
    /// One item
    pub one: String,
    /// Two items (optional, for languages with dual form)
    #[serde(default)]
    pub two: Option<String>,
    /// Few items (optional, for languages with paucal form)
    #[serde(default)]
    pub few: Option<String>,
    /// Many items (optional)
    #[serde(default)]
    pub many: Option<String>,
    /// Other (default plural)
    pub other: String,
}

/// Localization manager
pub struct Localization {
    current_locale: String,
    fallback_locale: String,
    strings: HashMap<String, HashMap<String, String>>,
    plurals: HashMap<String, HashMap<String, PluralForms>>,
    locale_names: HashMap<String, String>,
}

impl Default for Localization {
    fn default() -> Self {
        Self::new("en")
    }
}

impl Localization {
    /// Create a new localization manager with a default locale
    pub fn new(default_locale: &str) -> Self {
        Self {
            current_locale: default_locale.to_string(),
            fallback_locale: default_locale.to_string(),
            strings: HashMap::new(),
            plurals: HashMap::new(),
            locale_names: HashMap::new(),
        }
    }

    /// Load a locale from a file
    ///
    /// # Arguments
    /// * `path` - Path to the locale JSON file
    pub fn load_locale(&mut self, path: impl AsRef<Path>) -> Result<(), LocaleError> {
        let data = std::fs::read_to_string(path)?;
        let locale_file: LocaleFile =
            serde_json::from_str(&data).map_err(|e| LocaleError::ParseError(e.to_string()))?;

        self.locale_names
            .insert(locale_file.locale.clone(), locale_file.name.clone());
        self.strings
            .insert(locale_file.locale.clone(), locale_file.strings);
        self.plurals
            .insert(locale_file.locale.clone(), locale_file.plurals);

        Ok(())
    }

    /// Load a locale from a string
    pub fn load_locale_from_str(&mut self, data: &str) -> Result<(), LocaleError> {
        let locale_file: LocaleFile =
            serde_json::from_str(data).map_err(|e| LocaleError::ParseError(e.to_string()))?;

        self.locale_names
            .insert(locale_file.locale.clone(), locale_file.name.clone());
        self.strings
            .insert(locale_file.locale.clone(), locale_file.strings);
        self.plurals
            .insert(locale_file.locale.clone(), locale_file.plurals);

        Ok(())
    }

    /// Set the current locale
    pub fn set_locale(&mut self, locale: &str) -> Result<(), LocaleError> {
        if !self.strings.contains_key(locale) {
            return Err(LocaleError::LocaleNotFound(locale.to_string()));
        }
        self.current_locale = locale.to_string();
        Ok(())
    }

    /// Get the current locale code
    pub fn get_locale(&self) -> &str {
        &self.current_locale
    }

    /// Set the fallback locale
    pub fn set_fallback(&mut self, locale: &str) {
        self.fallback_locale = locale.to_string();
    }

    /// Get all available locales
    pub fn available_locales(&self) -> Vec<&str> {
        self.strings.keys().map(String::as_str).collect()
    }

    /// Get locale display name
    pub fn locale_name(&self, locale: &str) -> Option<&str> {
        self.locale_names.get(locale).map(String::as_str)
    }

    /// Get a translation by key
    ///
    /// Falls back to fallback locale if key not found in current locale.
    /// Returns the key itself if not found in any locale.
    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        // Try current locale
        if let Some(strings) = self.strings.get(&self.current_locale) {
            if let Some(value) = strings.get(key) {
                return value;
            }
        }

        // Try fallback locale
        if let Some(strings) = self.strings.get(&self.fallback_locale) {
            if let Some(value) = strings.get(key) {
                return value;
            }
        }

        // Return key as-is
        key
    }

    /// Get a translation with placeholder substitution
    ///
    /// # Arguments
    /// * `key` - Translation key
    /// * `args` - Key-value pairs for substitution: `&[("name", "value")]`
    ///
    /// Placeholders in the format `{name}` will be replaced.
    pub fn get_formatted(&self, key: &str, args: &[(&str, &str)]) -> String {
        let template = self.get(key);
        let mut result = template.to_string();

        for (name, value) in args {
            let placeholder = format!("{{{name}}}");
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Get a plural form translation
    ///
    /// # Arguments
    /// * `key` - Translation key
    /// * `count` - Number for determining plural form
    ///
    /// The appropriate plural form is chosen based on count.
    pub fn get_plural(&self, key: &str, count: u32) -> String {
        // Try current locale
        if let Some(plurals) = self.plurals.get(&self.current_locale) {
            if let Some(forms) = plurals.get(key) {
                return Self::select_plural_form(forms, count);
            }
        }

        // Try fallback locale
        if let Some(plurals) = self.plurals.get(&self.fallback_locale) {
            if let Some(forms) = plurals.get(key) {
                return Self::select_plural_form(forms, count);
            }
        }

        // Return key with count
        format!("{key}: {count}")
    }

    /// Get plural form with formatting
    pub fn get_plural_formatted(&self, key: &str, count: u32, args: &[(&str, &str)]) -> String {
        let template = self.get_plural(key, count);
        let mut result = template.replace("{count}", &count.to_string());

        for (name, value) in args {
            let placeholder = format!("{{{name}}}");
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Select the appropriate plural form based on count
    fn select_plural_form(forms: &PluralForms, count: u32) -> String {
        match count {
            0 => forms.zero.clone().unwrap_or_else(|| forms.other.clone()),
            1 => forms.one.clone(),
            2 => forms.two.clone().unwrap_or_else(|| forms.other.clone()),
            3..=4 => forms.few.clone().unwrap_or_else(|| forms.other.clone()),
            5..=10 => forms.many.clone().unwrap_or_else(|| forms.other.clone()),
            _ => forms.other.clone(),
        }
    }

    /// Check if a key exists in the current locale
    pub fn has_key(&self, key: &str) -> bool {
        self.strings
            .get(&self.current_locale)
            .is_some_and(|s| s.contains_key(key))
    }

    /// Get all keys in the current locale
    pub fn keys(&self) -> Vec<&str> {
        self.strings
            .get(&self.current_locale)
            .map(|s| s.keys().map(String::as_str).collect())
            .unwrap_or_default()
    }
}

/// Convenience macro for getting translations
///
/// # Examples
/// ```ignore
/// let text = t!(loc, "greeting");
/// let text = t!(loc, "welcome", ("name", "Alice"));
/// ```
#[macro_export]
macro_rules! t {
    ($loc:expr, $key:literal) => {
        $loc.get($key)
    };
    ($loc:expr, $key:literal, $(($arg_name:expr, $arg_val:expr)),* $(,)?) => {
        $loc.get_formatted($key, &[$(($arg_name, $arg_val)),*])
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_localization() -> Localization {
        let mut loc = Localization::new("en");

        let en_locale = r#"{
            "locale": "en",
            "name": "English",
            "strings": {
                "greeting": "Hello",
                "welcome": "Welcome, {name}!",
                "goodbye": "Goodbye"
            },
            "plurals": {
                "items": {
                    "one": "{count} item",
                    "other": "{count} items"
                }
            }
        }"#;

        let es_locale = r#"{
            "locale": "es",
            "name": "Español",
            "strings": {
                "greeting": "Hola",
                "welcome": "Bienvenido, {name}!",
                "goodbye": "Adiós"
            },
            "plurals": {
                "items": {
                    "one": "{count} artículo",
                    "other": "{count} artículos"
                }
            }
        }"#;

        loc.load_locale_from_str(en_locale).expect("load en locale");
        loc.load_locale_from_str(es_locale).expect("load es locale");

        loc
    }

    #[test]
    fn test_basic_translation() {
        let loc = create_test_localization();
        assert_eq!(loc.get("greeting"), "Hello");
    }

    #[test]
    fn test_formatted_translation() {
        let loc = create_test_localization();
        let result = loc.get_formatted("welcome", &[("name", "Alice")]);
        assert_eq!(result, "Welcome, Alice!");
    }

    #[test]
    fn test_language_switch() {
        let mut loc = create_test_localization();
        assert_eq!(loc.get("greeting"), "Hello");

        loc.set_locale("es").expect("set locale");
        assert_eq!(loc.get("greeting"), "Hola");
    }

    #[test]
    fn test_fallback() {
        let mut loc = create_test_localization();
        loc.set_locale("es").expect("set locale");

        // "nonexistent" doesn't exist, should return key
        assert_eq!(loc.get("nonexistent"), "nonexistent");
    }

    #[test]
    fn test_plural_forms() {
        let loc = create_test_localization();
        assert_eq!(loc.get_plural("items", 1), "{count} item");
        assert_eq!(loc.get_plural("items", 5), "{count} items");
    }

    #[test]
    fn test_plural_formatted() {
        let loc = create_test_localization();
        assert_eq!(loc.get_plural_formatted("items", 1, &[]), "1 item");
        assert_eq!(loc.get_plural_formatted("items", 5, &[]), "5 items");
    }

    #[test]
    fn test_available_locales() {
        let loc = create_test_localization();
        let locales = loc.available_locales();
        assert!(locales.contains(&"en"));
        assert!(locales.contains(&"es"));
    }

    #[test]
    fn test_t_macro() {
        let loc = create_test_localization();
        assert_eq!(t!(loc, "greeting"), "Hello");
        assert_eq!(t!(loc, "welcome", ("name", "Bob")), "Welcome, Bob!");
    }
}
