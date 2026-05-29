import { useEffect, useState } from "react";
import { getCheckInterval, setCheckInterval } from "../api";

/** Editable check interval in minutes. */
export default function IntervalSettings() {
  const [value, setValue] = useState("");
  const [saved, setSaved] = useState<number | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    getCheckInterval()
      .then((m) => {
        setSaved(m);
        setValue(String(m));
      })
      .catch(() => {});
  }, []);

  const parsed = Number(value);
  const valid = Number.isInteger(parsed) && parsed >= 1;
  const dirty = saved === null || parsed !== saved;

  async function save() {
    if (!valid) return;
    setBusy(true);
    try {
      await setCheckInterval(parsed);
      setSaved(parsed);
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="interval">
      <h2>Check interval</h2>
      <div className="interval-row">
        <input
          type="number"
          min={1}
          step={1}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          disabled={busy}
        />
        <span className="muted">minutes</span>
        <button
          type="button"
          onClick={save}
          disabled={busy || !valid || !dirty}
        >
          Save
        </button>
      </div>
    </section>
  );
}
