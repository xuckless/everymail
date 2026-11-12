import "virtual:tokens.css";
import "@everymail/design/components.css";
import "./ui/ui.css";
import "./chrome/chrome.css";
import { createRoot } from "react-dom/client";
import { connectEngine } from "./engine";
import { watchPresses } from "./presentation";
import { App } from "./shell";

watchPresses();

const container = document.createElement("div");
container.id = "root";
const root = createRoot(document.body.appendChild(container));
const engine = connectEngine((bytes) =>
	root.render(<App bytes={bytes} engine={engine} />),
);
