import { must, show } from "./show";

type Phase = "idle" | "sorting" | "outcomes" | "done";
const timeline: [Phase, number][] = [
	["sorting", 0],
	["outcomes", 1400],
	["done", 2400],
];

const demo = must<HTMLElement>("#demo");
const run = must<HTMLButtonElement>("button", demo);

const render = (phase: Phase) => {
	show(demo, phase);
	run.disabled = phase === "sorting" || phase === "outcomes";
	run.classList.toggle("pulse", phase === "done");
	for (const row of demo.querySelectorAll(".mrow:not(.keep)")) {
		row.classList.toggle("row-out", phase === "done");
	}
};

run.addEventListener("click", () => {
	for (const [phase, delay] of timeline) setTimeout(render, delay, phase);
});
