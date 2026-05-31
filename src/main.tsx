import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./utils/i18n";
import "./style.css";
import App from "./App";

document.addEventListener("contextmenu", (e) => e.preventDefault());

const root = document.getElementById("app");
if (!root) throw new Error("#app element not found");

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>
);
