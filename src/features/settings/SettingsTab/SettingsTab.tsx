import AutostartSettings from "@/features/settings/AutostartSettings/AutostartSettings";
import TokenSettings from "@/features/settings/TokenSettings/TokenSettings";
import UpdateSettings from "@/features/settings/UpdateSettings/UpdateSettings";

/** Settings tab: token, update checks (interval / startup / app updates), autostart. */
export default function SettingsTab() {
  return (
    <div className="tab-panel">
      <TokenSettings />
      <UpdateSettings />
      <AutostartSettings />
    </div>
  );
}
