import { type ReactNode, useContext } from "react";
import { TabsContext } from "@/components/ui/Tabs/Tabs";

interface Props<T extends string> {
  value: T;
  children: ReactNode;
}

/** A single tab button. Reads active state from the surrounding <Tabs>. */
export default function Tab<T extends string>({ value, children }: Props<T>) {
  const ctx = useContext(TabsContext);
  if (!ctx) {
    throw new Error("Tab must be used within <Tabs>");
  }
  const active = ctx.active === value;
  return (
    <button
      type="button"
      className={active ? "tab active" : "tab"}
      onClick={() => ctx.onChange(value)}
    >
      {children}
    </button>
  );
}
