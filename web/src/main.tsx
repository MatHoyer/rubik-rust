import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { engineReady } from "./engine";
import "./styles.css";

engineReady.then(() => {
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
});
