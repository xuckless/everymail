// Timings for the few animations Motion drives, read from the design tokens (never copied): the
// CSS variables are loaded before any component module runs (main.tsx imports them first).

const root = getComputedStyle(document.documentElement);

function timing(duration: string, easing: string) {
	const [x1 = 0, y1 = 0, x2 = 1, y2 = 1] = (
		root.getPropertyValue(easing).match(/-?[\d.]+/g) ?? []
	).map(Number);
	return {
		duration: Number.parseFloat(root.getPropertyValue(duration)) / 1000,
		ease: [x1, y1, x2, y2] as [number, number, number, number],
	};
}

// Entrances ease out; exits are 20% shorter and ease in; moves ease in and out.
export const enter = timing("--t-in", "--ease-out");
export const exit = timing("--t-out", "--ease-exit");
export const move = timing("--t-move", "--ease-in-out");
