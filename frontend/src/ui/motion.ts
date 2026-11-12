// Timings for the few animations Motion drives, from the design tokens (design/tokens.json, the one
// source). Under prefers-reduced-motion every duration is the tokens' reduced value, as the CSS
// variables are (design/plugin.ts), so Motion and CSS shorten together.
import tokens from "@everymail/design/tokens.json";
import { useReducedMotion } from "motion/react";

const value = (
	group: { tokens: { name: string; value: unknown }[] },
	name: string,
) => {
	const token = group.tokens.find((each) => each.name === name);
	if (typeof token?.value !== "string")
		throw new Error(`design token ${name} is missing`);
	return token.value;
};

function timing(duration: string, easing: string, reduced: boolean) {
	const ms = Number.parseFloat(
		reduced ? tokens.duration.reduced : value(tokens.duration, duration),
	);
	const [x1 = 0, y1 = 0, x2 = 1, y2 = 1] = (
		value(tokens.easing, easing).match(/-?[\d.]+/g) ?? []
	).map(Number);
	return {
		duration: ms / 1000,
		ease: [x1, y1, x2, y2] as [number, number, number, number],
	};
}

// Entrances ease out; exits are 20% shorter and ease in; moves ease in and out.
export function useTiming() {
	const reduced = useReducedMotion() === true;
	return {
		enter: timing("t-in", "ease-out", reduced),
		exit: timing("t-out", "ease-exit", reduced),
		move: timing("t-move", "ease-in-out", reduced),
		// The theme's circular reveal, in Web Animations terms (presentation.ts, showTheme).
		reveal: {
			duration: Number.parseFloat(
				reduced ? tokens.duration.reduced : value(tokens.duration, "t-reveal"),
			),
			easing: value(tokens.easing, "ease-in-out"),
		},
	};
}
