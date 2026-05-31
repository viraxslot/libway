export const en = {
  header: {
    title: "libway",
    markAllAsRead: "Mark all as read",
    checkNow: "Check now",
    checking: "Checking…",
  },
  tabs: {
    repositories: "Repositories",
    tags: "Tags",
    settings: "Settings",
  },
  repos: {
    addFormPlaceholder: "owner/repo",
    add: "Add",
    adding: "Adding…",
    addTag: "+ tag",
    tag: "(tag)",
    new: "new",
    notCheckedYet: "not checked yet",
    removeFromList: "Remove from list",
    searchRepos: "Search repositories…",
    noMatch: "No repositories match",
    openReleasePage: "Open release page",
    lastChecked: "Last checked",
    remove: "Remove",
    removeTagAria: "Remove tag {{tag}}",
    // Remove dialog
    removeTitle: "Remove repository",
    removeMessage1: "Stop tracking",
    confirmRemove: "Remove",
    cancelRemove: "Cancel",
  },
  tags: {
    // List
    noTagsYet: "No tags yet. Add tags to repositories first.",
    repoCount_one: "{{count}} repo",
    repoCount_other: "{{count}} repos",
    // Remove dialog
    removeTitle: "Remove tag",
    removeTagFromEverywhere: "Remove tag from all repositories",
    removeMessage: 'Remove "{{tag}}" from {{count}} repositories?',
    removeTag: "Remove tag",
    renameTag: "Rename tag",
    confirmRemove: "Remove",
    cancelRemove: "Cancel",
    // Merge dialog
    mergeTitle: "Merge tags",
    mergeMessage:
      'Merge "{{from}}" into "{{to}}"? {{count}} repositories will be affected.',
    confirmMerge: "Merge",
    cancelMerge: "Cancel",
  },
  settings: {
    // Github token
    tokenHeader: "Github token",
    tokenDescription:
      "Stored in the Keychain. Used for higher API rate limits.",
    tokenSaved: "Token saved.",
    tokenNotSet: "No token set.",
    saveToken: "Save",
    removeToken: "Remove",
    // Update settings
    updateHeader: "Update settings",
    saveInterval: "Save",
    intervalLabel: "minutes",
    checkOnStartup: "Check on startup",
    checkAppUpdates: "Check for app updates",
    // System settings
    systemHeader: "System",
    launchAtLogin: "Launch at login",
    languageLabel: "Language",
    english: "English",
    russian: "Russian",
    systemLanguage: "System",
  },
} as const;

// CLDR plural-category suffixes. Keys ending in one of these are pluralized
// and vary per language (English: _one/_other; Russian: _one/_few/_many), so
// they are exempt from the strict key-parity check below.
type PluralSuffix = "zero" | "one" | "two" | "few" | "many" | "other";

type IsPlural<K> = K extends `${string}_${PluralSuffix}` ? true : false;

// Non-plural keys of a group must match exactly across locales; plural keys are
// optional (each locale supplies the forms its grammar needs).
type Group<G> = {
  [P in keyof G as IsPlural<P> extends true ? never : P]: string;
} & {
  [suffix: `${string}_${PluralSuffix}`]: string;
};

export type LanguageDict = {
  [K in keyof typeof en]: Group<(typeof en)[K]>;
};
