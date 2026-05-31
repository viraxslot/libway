import App from "@/App";
import "@/i18n";

import React from "react";
import ReactDOM from "react-dom/client";
import { getLanguage } from "@/api";
import i18n from "@/i18n";
import "./styles.css";

async function bootstrap() {
  try {
    const lang = await getLanguage();
    await i18n.changeLanguage(lang);
  } catch (err) {
    console.error("Failed to load language, falling back to en", err);
  }

  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}

void bootstrap();
