use crate::db::{get_setting, Db, SettingKey};
use sys_locale::get_locale;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Language {
    En,
    Ru,
}

/// CLDR plural category. English uses One/Other; Russian uses One/Few/Many.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum PluralForm {
    One,
    Few,
    Many,
}

/// Pick the Russian plural form for `n` per CLDR rules:
/// - one:  n % 10 == 1 and n % 100 != 11        (1, 21, 31, … but not 11)
/// - few:  n % 10 in 2..=4 and n % 100 not 12..=14 (2, 3, 4, 22, … but not 12-14)
/// - many: everything else                       (0, 5..=20, 11..=14, 25, …)
fn ru_plural(n: u64) -> PluralForm {
    let m10 = n % 10;
    let m100 = n % 100;
    match (m10, m100) {
        (1, m100) if m100 != 11 => PluralForm::One,
        (2..=4, m100) if !(12..=14).contains(&m100) => PluralForm::Few,
        _ => PluralForm::Many,
    }
}

impl Language {
    pub fn parse(s: &str) -> Language {
        match s {
            "ru" => Language::Ru,
            _ => Language::En,
        }
    }

    pub fn as_code(&self) -> &str {
        match self {
            Language::En => "en",
            Language::Ru => "ru",
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

/// Localized strings for the native UI (tray menu + notifications).
///
/// The frontend uses i18next; the small set of native strings is kept here as
/// plain `match` functions to avoid pulling a runtime i18n engine into Rust.
/// Static strings return `&'static str`; strings with interpolation or
/// pluralization return `String`.
pub mod tr {
    use super::{ru_plural, Language, PluralForm};

    pub fn ungrouped(lang: Language) -> &'static str {
        match lang {
            Language::En => "Ungrouped",
            Language::Ru => "Без группы",
        }
    }

    pub fn all_up_to_date(lang: Language) -> &'static str {
        match lang {
            Language::En => "All up to date",
            Language::Ru => "Всё обновлено",
        }
    }

    pub fn no_repositories(lang: Language) -> &'static str {
        match lang {
            Language::En => "No repositories",
            Language::Ru => "Нет репозиториев",
        }
    }

    pub fn not_checked_yet(lang: Language) -> &'static str {
        match lang {
            Language::En => "not checked yet",
            Language::Ru => "ещё не проверялось",
        }
    }

    pub fn just_now(lang: Language) -> &'static str {
        match lang {
            Language::En => "just now",
            Language::Ru => "только что",
        }
    }

    pub fn check_now(lang: Language) -> &'static str {
        match lang {
            Language::En => "Check now",
            Language::Ru => "Проверить сейчас",
        }
    }

    pub fn mark_all_as_read(lang: Language) -> &'static str {
        match lang {
            Language::En => "Mark all as read",
            Language::Ru => "Отметить всё прочитанным",
        }
    }

    pub fn settings(lang: Language) -> &'static str {
        match lang {
            Language::En => "Settings…",
            Language::Ru => "Настройки…",
        }
    }

    pub fn quit(lang: Language) -> &'static str {
        match lang {
            Language::En => "Quit",
            Language::Ru => "Выход",
        }
    }

    pub fn about(lang: Language) -> &'static str {
        match lang {
            Language::En => "About",
            Language::Ru => "О программе",
        }
    }

    pub fn view_on_github(lang: Language) -> &'static str {
        match lang {
            Language::En => "↗ View on GitHub",
            Language::Ru => "↗ Открыть на GitHub",
        }
    }

    pub fn notify_all_up_to_date(lang: Language) -> &'static str {
        match lang {
            Language::En => "All repositories are up to date.",
            Language::Ru => "Все репозитории обновлены.",
        }
    }

    /// Status line head: "N updates" with pluralization.
    pub fn updates_count(lang: Language, n: u64) -> String {
        match lang {
            Language::En => {
                if n == 1 {
                    format!("{n} update")
                } else {
                    format!("{n} updates")
                }
            }
            Language::Ru => match ru_plural(n) {
                PluralForm::One => format!("{n} обновление"),
                PluralForm::Few => format!("{n} обновления"),
                PluralForm::Many => format!("{n} обновлений"),
            },
        }
    }

    /// "N updates found." notification body (manual check result).
    pub fn updates_found(lang: Language, n: u64) -> String {
        match lang {
            Language::En => format!("{n} updates found."),
            Language::Ru => match ru_plural(n) {
                PluralForm::One => format!("Найдено {n} обновление."),
                PluralForm::Few => format!("Найдено {n} обновления."),
                PluralForm::Many => format!("Найдено {n} обновлений."),
            },
        }
    }

    pub fn checked_ago(lang: Language, relative: &str) -> String {
        match lang {
            Language::En => format!("checked {relative}"),
            Language::Ru => format!("проверено {relative}"),
        }
    }

    pub fn minutes_ago(lang: Language, n: i64) -> String {
        match lang {
            Language::En => format!("{n}m ago"),
            Language::Ru => format!("{n} мин назад"),
        }
    }

    pub fn hours_ago(lang: Language, n: i64) -> String {
        match lang {
            Language::En => format!("{n}h ago"),
            Language::Ru => format!("{n} ч назад"),
        }
    }

    pub fn days_ago(lang: Language, n: i64) -> String {
        match lang {
            Language::En => format!("{n}d ago"),
            Language::Ru => format!("{n} дн назад"),
        }
    }

    pub fn update_available(lang: Language, version: &str) -> String {
        match lang {
            Language::En => format!("↗ Update available: {version}"),
            Language::Ru => format!("↗ Доступно обновление: {version}"),
        }
    }

    pub fn new_version(lang: Language, version: &str) -> String {
        match lang {
            Language::En => format!("New version: {version}"),
            Language::Ru => format!("Новая версия: {version}"),
        }
    }

    pub fn update_found(lang: Language, repo: &str) -> String {
        match lang {
            Language::En => format!("Update found: {repo}"),
            Language::Ru => format!("Найдено обновление: {repo}"),
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

    #[test]
    fn ru_plural_one_form() {
        // n % 10 == 1, except 11.
        for n in [1, 21, 31, 101, 1001] {
            assert_eq!(ru_plural(n), PluralForm::One, "n = {n}");
        }
    }

    #[test]
    fn ru_plural_few_form() {
        // n % 10 in 2..=4, except 12..=14.
        for n in [2, 3, 4, 22, 23, 24, 102, 1002] {
            assert_eq!(ru_plural(n), PluralForm::Few, "n = {n}");
        }
    }

    #[test]
    fn ru_plural_many_form() {
        // 0, 5..=20, the 11..=14 teens, and 25..=30.
        for n in [0, 5, 9, 10, 11, 12, 13, 14, 15, 19, 20, 25, 100, 111] {
            assert_eq!(ru_plural(n), PluralForm::Many, "n = {n}");
        }
    }

    #[test]
    fn updates_count_english_pluralizes() {
        assert_eq!(tr::updates_count(Language::En, 1), "1 update");
        assert_eq!(tr::updates_count(Language::En, 5), "5 updates");
    }

    #[test]
    fn updates_count_russian_pluralizes() {
        assert_eq!(tr::updates_count(Language::Ru, 1), "1 обновление");
        assert_eq!(tr::updates_count(Language::Ru, 3), "3 обновления");
        assert_eq!(tr::updates_count(Language::Ru, 5), "5 обновлений");
        // The 11..=14 teens take the "many" form even though they end in 1..=4.
        assert_eq!(tr::updates_count(Language::Ru, 11), "11 обновлений");
        assert_eq!(tr::updates_count(Language::Ru, 12), "12 обновлений");
    }
}
