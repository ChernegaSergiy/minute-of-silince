import "./style.css";
import { App } from "./app";

const root = document.getElementById("app");
if (!root) throw new Error("#app element not found");

const app = new App(root);
app.mount();
