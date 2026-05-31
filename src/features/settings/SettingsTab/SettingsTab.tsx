import SystemSettings from "@/features/settings/SystemSettings/SystemSettings";
import TokenSettings from "@/features/settings/TokenSettings/TokenSettings";
import UpdateSettings from "@/features/settings/UpdateSettings/UpdateSettings";

/** Settings tab: token, update checks (interval / startup / app updates), autostart. */
export default function SettingsTab() {
  return (
    <div className="tab-panel">
      <TokenSettings />
      <UpdateSettings />
      <SystemSettings />
    </div>
  );
}
