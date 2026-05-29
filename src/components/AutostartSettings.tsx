import { useEffect, useState } from "react";
import { getAutostart, setAutostart } from "../api";

/** Toggle for launching the app at login. */
export default function AutostartSettings() {
  const [enabled, setEnabled] = useState(false);

  useEffect(() => {
    getAutostart()
      .then(setEnabled)
      .catch(() => {});
  }, []);

  async function toggle() {
    const next = !enabled;
    await setAutostart(next);
    setEnabled(next);
  }

  return (
    <section className="autostart">
      <label className="checkbox-row">
        <input type="checkbox" checked={enabled} onChange={toggle} />
        Launch at login
      </label>
    </section>
  );
}
