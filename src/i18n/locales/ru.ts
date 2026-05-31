import type { LanguageDict } from "@/i18n/locales/en";

export const ru = {
  header: {
    title: "libway",
    markAllAsRead: "Отметить всё прочитанным",
    checkNow: "Проверить сейчас",
    checking: "Проверяем…",
  },
  tabs: {
    repositories: "Репозитории",
    tags: "Теги",
    settings: "Настройки",
  },
  repos: {
    addFormPlaceholder: "владелец/репозиторий",
    add: "Добавить",
    adding: "Добавляем…",
    addTag: "+ тег",
    tag: "(тег)",
    new: "новый",
    notCheckedYet: "ещё не проверен",
    removeFromList: "Удалить из списка",
    searchRepos: "Искать репозитории…",
    noMatch: "Репозитории не найдены",
    openReleasePage: "Открыть страницу релиза",
    lastChecked: "Последняя проверка",
    remove: "Удалить",
    removeTagAria: "Удалить тег {{tag}}",
    // Remove dialog
    removeTitle: "Удалить репозиторий",
    removeMessage1: "Перестать отслеживать",
    confirmRemove: "Удалить",
    cancelRemove: "Отмена",
  },
  tags: {
    // List
    noTagsYet:
      "Пока что нет тегов. Сначала добавьте хотя бы один тег к репозиториям.",
    repoCount_one: "{{count}} репозиторий",
    repoCount_few: "{{count}} репозитория",
    repoCount_many: "{{count}} репозиториев",
    // Remove dialog
    removeTitle: "Удалить тег",
    removeTagFromEverywhere: "Удалить тег из всех репозиториев",
    removeTag: "Удалить тег",
    renameTag: "Переименовать тег",
    removeMessage: "Удалить «{{tag}}» из репозиториев ({{count}})?",
    confirmRemove: "Удалить",
    cancelRemove: "Отмена",
    // Merge dialog
    mergeTitle: "Объединить теги",
    mergeMessage:
      "Объединить «{{from}}» с «{{to}}»? Это затронет репозиториев: {{count}}.",
    confirmMerge: "Добавить",
    cancelMerge: "Отмена",
  },
  settings: {
    // Github token
    tokenHeader: "Токен для Github",
    tokenDescription:
      "Хранится в Keychain. Используется для обхода лимитов по API-запросам.",
    tokenSaved: "Токен сохранён.",
    tokenNotSet: "Токен не установлен.",
    saveToken: "Сохранить",
    removeToken: "Удалить",
    // Update settings
    updateHeader: "Настройки обновления",
    saveInterval: "Сохранить",
    intervalLabel: "минут",
    checkOnStartup: "Проверять при старте приложения",
    checkAppUpdates: "Проверять обновления libway",
    // System settings
    systemHeader: "Системные",
    launchAtLogin: "Запускать при старте системы",
    languageLabel: "Язык",
    english: "Английский",
    russian: "Русский",
    systemLanguage: "Системный",
  },
} satisfies LanguageDict;
