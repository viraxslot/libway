import AutostartSettings from "./AutostartSettings";
import IntervalSettings from "./IntervalSettings";
import TokenSettings from "./TokenSettings";

/** Settings tab: token, check interval / startup, autostart. */
export default function SettingsTab() {
  return (
    <div className="tab-panel">
      <TokenSettings />
      <IntervalSettings />
      <AutostartSettings />
    </div>
  );
}
