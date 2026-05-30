import { useEffect, useState } from "react";
import {
  getCheckInterval,
  getCheckOnStartup,
  setCheckInterval,
  setCheckOnStartup,
} from "@/api";

/** Editable check interval in minutes plus the "check on startup" toggle. */
export default function IntervalSettings() {
  const [value, setValue] = useState("");
  const [saved, setSaved] = useState<number | null>(null);
  const [busy, setBusy] = useState(false);
  const [onStartup, setOnStartup] = useState(true);

  useEffect(() => {
    getCheckInterval()
      .then((m) => {
        setSaved(m);
        setValue(String(m));
      })
      .catch(() => {});
    getCheckOnStartup()
      .then(setOnStartup)
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

  async function toggleOnStartup() {
    const next = !onStartup;
    await setCheckOnStartup(next);
    setOnStartup(next);
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
      <label className="checkbox-row">
        <input type="checkbox" checked={onStartup} onChange={toggleOnStartup} />
        Check on startup
      </label>
    </section>
  );
}
