import AutostartSettings from "@/features/settings/AutostartSettings/AutostartSettings";
import IntervalSettings from "@/features/settings/IntervalSettings/IntervalSettings";
import TokenSettings from "@/features/settings/TokenSettings/TokenSettings";

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
