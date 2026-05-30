import { useEffect, useState } from "react";
import { getAutostart, setAutostart } from "@/api";
import Checkbox from "@/components/ui/Checkbox/Checkbox";

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
      <Checkbox checked={enabled} onChange={toggle} label="Launch at login" />
    </section>
  );
}
