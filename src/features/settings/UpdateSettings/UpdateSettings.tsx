import { useEffect, useState } from "react";
import {
  getCheckInterval,
  getCheckOnStartup,
  getCheckSelfUpdate,
  setCheckInterval,
  setCheckOnStartup,
  setCheckSelfUpdate,
} from "@/api";
import Button from "@/components/ui/Button/Button";
import Checkbox from "@/components/ui/Checkbox/Checkbox";
import Input from "@/components/ui/Input/Input";

/** Update-check settings: the interval, "check on startup", and the
 * "check for app updates" toggle. */
export default function UpdateSettings() {
  const [value, setValue] = useState("");
  const [saved, setSaved] = useState<number | null>(null);
  const [busy, setBusy] = useState(false);
  const [onStartup, setOnStartup] = useState(true);
  const [selfUpdate, setSelfUpdate] = useState(true);

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
    getCheckSelfUpdate()
      .then(setSelfUpdate)
      .catch(() => {});
  }, []);

  const parsed = Number(value);
  const valid = Number.isInteger(parsed) && parsed >= 1;
  const dirty = saved === null || parsed !== saved;

  async function save() {
    if (!valid) {
      return;
    }
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

  async function toggleSelfUpdate() {
    const next = !selfUpdate;
    await setCheckSelfUpdate(next);
    setSelfUpdate(next);
  }

  return (
    <section className="interval">
      <h2>Check interval</h2>
      <div className="interval-row">
        <Input
          type="number"
          min={1}
          step={1}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          disabled={busy}
        />
        <span className="muted">minutes</span>
        <Button
          type="button"
          onClick={save}
          disabled={busy || !valid || !dirty}
        >
          Save
        </Button>
      </div>
      <Checkbox
        checked={onStartup}
        onChange={toggleOnStartup}
        label="Check on startup"
      />
      <Checkbox
        checked={selfUpdate}
        onChange={toggleSelfUpdate}
        label="Check for app updates"
      />
    </section>
  );
}
