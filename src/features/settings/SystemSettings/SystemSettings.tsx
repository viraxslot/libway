import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { getAutostart, getLanguage, setAutostart, setLanguage } from "@/api";
import Checkbox from "@/components/ui/Checkbox/Checkbox";
import Dropdown from "@/components/ui/Dropdown/Dropdown";
import i18n from "@/i18n";
import type { LanguageChoice } from "@/types";

/** Toggle for launching the app at login. */
export default function SystemSettings() {
  const [enabled, setEnabled] = useState(false);
  const [language, setUiLanguage] = useState<LanguageChoice>("system");
  const { t } = useTranslation();

  useEffect(() => {
    getAutostart()
      .then(setEnabled)
      .catch(() => {});

    getLanguage()
      .then(setUiLanguage)
      .catch(() => {});
  }, []);

  async function toggle() {
    const next = !enabled;
    await setAutostart(next);
    setEnabled(next);
  }

  async function handleLanguageChange(value: LanguageChoice) {
    await setLanguage(value);
    const effective = await getLanguage();
    await i18n.changeLanguage(effective);
    setUiLanguage(value);
  }

  return (
    <section className="autostart">
      <h2>{t("settings.systemHeader")}</h2>
      <Checkbox
        checked={enabled}
        onChange={toggle}
        label={t("settings.launchAtLogin")}
      />
      <div className="language-field">
        <span>{t("settings.languageLabel")}</span>
        <Dropdown<LanguageChoice>
          value={language}
          options={[
            { value: "system", label: t("settings.systemLanguage") },
            { value: "en", label: t("settings.english") },
            { value: "ru", label: t("settings.russian") },
          ]}
          onChange={handleLanguageChange}
        />
      </div>
    </section>
  );
}
