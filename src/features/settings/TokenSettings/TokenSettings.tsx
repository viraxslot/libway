import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { clearToken, hasToken, setToken } from "@/api";
import Button from "@/components/ui/Button/Button";
import Input from "@/components/ui/Input/Input";

/** GitHub token input. The value is stored in the macOS Keychain, never shown. */
export default function TokenSettings() {
  const [stored, setStored] = useState(false);
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);
  const { t } = useTranslation();

  useEffect(() => {
    hasToken()
      .then(setStored)
      .catch(() => setStored(false));
  }, []);

  async function save() {
    setBusy(true);
    try {
      await setToken(value);
      setValue("");
      setStored(await hasToken());
    } finally {
      setBusy(false);
    }
  }

  async function remove() {
    setBusy(true);
    try {
      await clearToken();
      setStored(false);
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="token">
      <h2>{t("settings.tokenHeader")}</h2>
      <p className="muted">
        {t("settings.tokenDescription")}
        {stored
          ? ` ${t("settings.tokenSaved")}`
          : ` ${t("settings.tokenNotSet")}`}
      </p>
      <div className="token-row">
        <Input
          type="password"
          placeholder={stored ? "••••••••" : "ghp_…"}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          disabled={busy}
          spellCheck={false}
        />
        <Button type="button" onClick={save} disabled={busy || !value.trim()}>
          {t("settings.saveToken")}
        </Button>
        {stored && (
          <Button
            variant="secondary"
            type="button"
            onClick={remove}
            disabled={busy}
          >
            {t("settings.removeToken")}
          </Button>
        )}
      </div>
    </section>
  );
}
