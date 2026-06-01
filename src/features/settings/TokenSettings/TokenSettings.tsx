import { useEffect, useState } from "react";
import { clearToken, hasToken, setToken } from "@/api";
import Button from "@/components/ui/Button/Button";
import ConfirmDialog from "@/components/ui/ConfirmDialog/ConfirmDialog";
import Input from "@/components/ui/Input/Input";

/** GitHub token input. The value is stored in the macOS Keychain, never shown. */
export default function TokenSettings() {
  const [stored, setStored] = useState(false);
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);
  const [confirming, setConfirming] = useState(false);

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
    setConfirming(false);
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
        <Input
          type="password"
          placeholder={stored ? "••••••••" : "ghp_…"}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          disabled={busy}
          spellCheck={false}
        />
        <Button type="button" onClick={save} disabled={busy || !value.trim()}>
          Save
        </Button>
        {stored && (
          <Button
            variant="secondary"
            type="button"
            onClick={() => setConfirming(true)}
            disabled={busy}
          >
            Remove
          </Button>
        )}
      </div>

      {confirming && (
        <ConfirmDialog
          title="Remove GitHub token"
          message="Remove the saved token from the Keychain? API requests will fall back to the lower unauthenticated rate limit."
          confirmLabel="Remove token"
          onConfirm={remove}
          onCancel={() => setConfirming(false)}
        />
      )}
    </section>
  );
}
