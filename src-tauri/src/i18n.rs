use crate::db::{get_setting, Db, SettingKey};
use sys_locale::get_locale;

#[derive(Eq, PartialEq, Debug)]
pub enum Language {
    En,
    Ru,
}

impl Language {
    pub fn parse(s: &str) -> Language {
        match s {
            "ru" => Language::Ru,
            _ => Language::En,
        }
    }

    fn from_system_locale(locale: &str) -> Language {
        let l = locale.split('-').next().unwrap_or("en");
        Language::parse(l)
    }

    pub fn system() -> Language {
        let locale = get_locale().unwrap_or_else(|| String::from("en-US"));
        Language::from_system_locale(&locale)
    }

    pub fn from_settings(db: &Db) -> Language {
        let setting = db
            .with(|c| get_setting(c, SettingKey::Language))
            .ok()
            .flatten();

        match setting.as_deref() {
            Some("system") => Language::system(),
            Some(s) => Language::parse(s),
            None => Language::system(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{set_setting, Db};
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_en_returns_en() {
        let lang: Language = Language::parse("en");
        assert_eq!(lang, Language::En);
    }

    #[test]
    fn parse_ru_returns_ru() {
        let lang: Language = Language::parse("ru");
        assert_eq!(lang, Language::Ru);
    }

    #[test]
    fn parse_unknown_returns_en() {
        let lang: Language = Language::parse("xx");
        assert_eq!(lang, Language::En);
    }

    #[test]
    fn returns_system_language_when_setting_is_not_set() {
        let db = Db::open_in_memory().unwrap();
        let system_locale = Language::system();

        let lang = Language::from_settings(&db);
        assert_eq!(lang, system_locale);
    }

    #[test]
    fn returns_system_language_when_setting_is_system() {
        let db = Db::open_in_memory().unwrap();
        db.with(|conn| set_setting(conn, SettingKey::Language, "system"))
            .unwrap();

        let system_locale = Language::system();
        let lang = Language::from_settings(&db);

        assert_eq!(lang, system_locale);
    }

    #[test]
    fn returns_en_language_when_setting_is_en() {
        let db = Db::open_in_memory().unwrap();
        db.with(|conn| set_setting(conn, SettingKey::Language, "en"))
            .unwrap();

        let lang = Language::from_settings(&db);
        assert_eq!(lang, Language::En);
    }

    #[test]
    fn returns_ru_language_when_setting_is_ru() {
        let db = Db::open_in_memory().unwrap();
        db.with(|conn| set_setting(conn, SettingKey::Language, "ru"))
            .unwrap();

        let lang = Language::from_settings(&db);
        assert_eq!(lang, Language::Ru);
    }

    #[test]
    fn parses_russian_locale_correctly() {
        let lang = Language::from_system_locale("ru-RU");
        assert_eq!(lang, Language::Ru);
    }

    #[test]
    fn parses_english_locale_correctly() {
        let lang = Language::from_system_locale("en-CA");
        assert_eq!(lang, Language::En);
    }

    #[test]
    fn returns_english_for_unsupported_locales() {
        let lang = Language::from_system_locale("de-GE");
        assert_eq!(lang, Language::En);
    }

    #[test]
    fn returns_default_when_unable_to_parse_locale() {
        let lang = Language::from_system_locale("");
        assert_eq!(lang, Language::En);
    }
}
