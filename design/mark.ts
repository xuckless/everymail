// assets/mark.svg, the standalone mark the README and the Tauri icons need, is generated from the
// `mark` symbol in logos.svg (the one source), with the dark theme's mark and cyan colours from
// tokens.json. `bun design/mark.ts` writes it; `--check` fails when it is out of date.
import tokens from "./tokens.json" with { type: "json" };

const sprite = await Bun.file(new URL("logos.svg", import.meta.url)).text();
const symbol = sprite.match(
	/<symbol id="mark" viewBox="([^"]+)">([\s\S]*?)<\/symbol>/,
);
if (!symbol) throw new Error("logos.svg has no mark symbol");
const colour = (name: string) => {
	const value = tokens.color.tokens.find((token) => token.name === name)?.value;
	if (typeof value !== "object") throw new Error(`no themed colour ${name}`);
	return value.dark;
};
const body = (symbol[2] ?? "")
	.replace('style="fill:var(--mark)"', `fill="${colour("mark")}"`)
	.replace('style="fill:var(--cyan)"', `fill="${colour("cyan")}"`);
const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${symbol[1]}"><title>EveryMail</title>${body}</svg>\n`;

const target = new URL("../assets/mark.svg", import.meta.url);
if (process.argv.includes("--check")) {
	if ((await Bun.file(target).text()) !== svg) {
		console.error(
			"assets/mark.svg is out of date with design/logos.svg: run bun design/mark.ts",
		);
		process.exit(1);
	}
} else await Bun.write(target, svg);
