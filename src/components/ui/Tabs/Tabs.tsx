import { createContext, type ReactNode } from "react";

interface TabsContextValue {
  active: string;
  onChange: (value: string) => void;
}

/** Shared by Tabs and Tab to coordinate the active tab. */
export const TabsContext = createContext<TabsContextValue | null>(null);

interface Props<T extends string> {
  value: T;
  onChange: (value: T) => void;
  children: ReactNode;
}

/**
 * Tab strip. Provides the active value and change handler to nested <Tab>s.
 * Generic over the tab id type so the caller's union is preserved end to end.
 */
export default function Tabs<T extends string>({
  value,
  onChange,
  children,
}: Props<T>) {
  return (
    <nav className="tabs">
      <TabsContext.Provider
        value={{ active: value, onChange: onChange as (value: string) => void }}
      >
        {children}
      </TabsContext.Provider>
    </nav>
  );
}
