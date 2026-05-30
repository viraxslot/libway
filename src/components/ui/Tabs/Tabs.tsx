import { createContext, type ReactNode } from "react";

interface TabsContextValue {
  active: string;
  onChange: (value: string) => void;
}

/** Shared by Tabs and Tab to coordinate the active tab. */
export const TabsContext = createContext<TabsContextValue | null>(null);

interface Props {
  value: string;
  onChange: (value: string) => void;
  children: ReactNode;
}

/** Tab strip. Provides the active value and change handler to nested <Tab>s. */
export default function Tabs({ value, onChange, children }: Props) {
  return (
    <nav className="tabs">
      <TabsContext.Provider value={{ active: value, onChange }}>
        {children}
      </TabsContext.Provider>
    </nav>
  );
}
