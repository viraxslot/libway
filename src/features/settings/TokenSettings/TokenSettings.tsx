import { useEffect, useState } from "react";
import { clearToken, hasToken, setToken } from "@/api";

/** GitHub token input. The value is stored in the macOS Keychain, never shown. */
export default function TokenSettings() {
  const [stored, setStored] = useState(false);
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);

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
      <h2>GitHub token</h2>
      <p className="muted">
        Stored in the Keychain. Used for higher API rate limits.
        {stored ? " Token saved." : " No token set."}
      </p>
      <div className="token-row">
        <input
          type="password"
          placeholder={stored ? "••••••••" : "ghp_…"}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          disabled={busy}
          spellCheck={false}
        />
        <button type="button" onClick={save} disabled={busy || !value.trim()}>
          Save
        </button>
        {stored && (
          <button
            type="button"
            className="secondary"
            onClick={remove}
            disabled={busy}
          >
            Remove
          </button>
        )}
      </div>
    </section>
  );
}
