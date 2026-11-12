import { fileURLToPath } from "node:url";
import { designPlugin } from "@everymail/design/plugin";
import react from "@vitejs/plugin-react";
import { defineConfig, type Plugin } from "vite";

// The body Service Worker (src/body-sw.ts) is served from /src/body-sw.ts in dev and built to
// /body-sw.js; both control the whole origin, so its responses allow the root scope.
const bodyWorker = (): Plugin => ({
	name: "body-worker",
	apply: "build",
	buildStart() {
		this.emitFile({
			type: "chunk",
			id: fileURLToPath(new URL("./src/body-sw.ts", import.meta.url)),
			fileName: "body-sw.js",
		});
	},
});

export default defineConfig({
	plugins: [react(), designPlugin(), bodyWorker()],
	clearScreen: false,
	resolve: { tsconfigPaths: true },
	worker: { format: "es" },
	server: {
		port: 5173,
		strictPort: true,
		headers: { "Service-Worker-Allowed": "/" },
		proxy: { "/api.v1.": "http://127.0.0.1:8787" },
	},
});
