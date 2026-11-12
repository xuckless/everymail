import type { Plugin } from "vite";
import tokens from "./tokens.json" with { type: "json" };

type Value = string | { dark: string; light: string };
const all = [
	tokens.color,
	tokens.spacing,
	tokens.radius,
	tokens.shadow,
	tokens.duration,
	tokens.easing,
].flatMap((group) => group.tokens as { name: string; value: Value }[]);

const declare = (pick: (value: Value) => string | undefined) =>
	all
		.flatMap(({ name, value }) => {
			const picked = pick(value);
			return picked === undefined ? [] : [`--${name}:${picked}`];
		})
		.join(";");
const dark = declare((v) => (typeof v === "string" ? v : v.dark));
const light = declare((v) => (typeof v === "string" ? undefined : v.light));
const families = Object.entries(tokens.type.families)
	.map(([name, family]) => `--font-${name}:${family}`)
	.join(";");

// Dark is the default, the system light preference overrides it, and an explicit data-theme overrides both.
const css = `:root{color-scheme:dark light;${dark};${families}}
@media (prefers-color-scheme:light){:root:not([data-theme=dark]){${light}}}
:root[data-theme=light]{color-scheme:light;${light}}
:root[data-theme=dark]{color-scheme:dark}`;

// Sprites are referenced through <use>, which does not work from a data: URL, so they are never inlined.
export const designPlugin = (): Plugin => ({
	name: "design",
	config: () => ({
		build: {
			assetsInlineLimit: (file) => (file.endsWith(".svg") ? false : undefined),
		},
	}),
	resolveId: (id) =>
		id === "virtual:tokens.css" ? "\0virtual:tokens.css" : undefined,
	load: (id) => (id === "\0virtual:tokens.css" ? css : undefined),
});
