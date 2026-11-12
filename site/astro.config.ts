import { designPlugin } from "@everymail/design/plugin";
import { defineConfig } from "astro/config";

export default defineConfig({
	server: { port: 4321 },
	devToolbar: { enabled: false },
	vite: { plugins: [designPlugin()] },
});
