import React from "react";
import ReactDOM from "react-dom/client";

import { App } from "./App";
import { applyTheme } from "./theme";
import "./styles/global.css";

applyTheme("system");

if (import.meta.env.PROD) {
  window.addEventListener("contextmenu", (e) => {
    const el = e.target as HTMLElement;
    if (!el.closest("input, textarea")) e.preventDefault();
  });
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
