import { queueNote } from "../states";
import { must, show } from "./show";

const agents = must<HTMLElement>("#agent-cards");
for (const radio of document.querySelectorAll<HTMLInputElement>(
	"[name=agent]",
)) {
	radio.addEventListener("change", () => show(agents, radio.value));
}

const perms = must<HTMLFormElement>("#perms");
const note = must<HTMLElement>("#queue-note");
perms.addEventListener("change", () => {
	const asks = perms.querySelectorAll<HTMLInputElement>("[value=ask]:checked");
	note.textContent = queueNote([...asks].map((ask) => ask.name));
	for (const animation of note.getAnimations()) animation.play();
});
