import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin } from "vite";
import tokens from "./tokens.json" with { type: "json" };

type Value = string | { dark: string; light: string };
const groups: { tokens: { name: string; value: Value }[] }[] = [
	tokens.color,
	tokens.spacing,
	tokens.radius,
	tokens.shadow,
	tokens.duration,
	tokens.easing,
];
const all = groups.flatMap((group) => group.tokens);
const declare = (pick: (value: Value) => string | undefined) =>
	all
		.flatMap(({ name, value }) => {
			const picked = pick(value);
			return picked === undefined ? [] : [`--${name}:${picked}`];
		})
		.join(";");
const families = Object.entries(tokens.type.families)
	.map(([name, family]) => `--font-${name}:${family}`)
	.join(";");

const tokensCss = `:root{color-scheme:dark light;${declare((v) => (typeof v === "string" ? v : v.dark))};${families}}
@media (prefers-color-scheme:light){:root{${declare((v) => (typeof v === "string" ? undefined : v.light))}}}`;

const tokensPlugin = (): Plugin => ({
	name: "tokens-css",
	resolveId: (id) =>
		id === "virtual:tokens.css" ? "\0virtual:tokens.css" : undefined,
	load: (id) => (id === "\0virtual:tokens.css" ? tokensCss : undefined),
});

export default defineConfig({
	plugins: [react(), tokensPlugin()],
	clearScreen: false,
	resolve: { tsconfigPaths: true },
	worker: { format: "es" },
	server: {
		port: 5173,
		strictPort: true,
		proxy: { "/api.v1.": "http://127.0.0.1:8787" },
	},
});
