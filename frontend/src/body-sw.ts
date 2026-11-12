// Web only: answers `/body/<request>` with the message body, an HTML document the tab's engine
// Worker serves as bytes (docs/ENGINE.md, Message bodies on web). The request is everything after
// `/body/`, query included (`m-slides-4?theme=dark`); the answer is the bytes and the engine's
// content policy. Never cached.
//
// The response is sandboxed by that policy, the same isolation as the iframe attribute
// sandbox="allow-scripts" without allow-same-origin: Chromium does not route the navigation of an
// iframe carrying that attribute through a Service Worker, so the body iframe on web has none.

import type { BodyAnswer } from "./body";

declare const self: ServiceWorkerGlobalScope;

self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (event) =>
	event.waitUntil(self.clients.claim()),
);

function ask(client: Client, request: string) {
	const channel = new MessageChannel();
	const answer = new Promise<BodyAnswer>((resolve) => {
		channel.port1.onmessage = (message) => resolve(message.data);
	});
	client.postMessage(request, [channel.port2]);
	return answer;
}

const headers = (policy: string) => ({
	"Content-Type": "text/html; charset=utf-8",
	"Cache-Control": "no-store",
	"Content-Security-Policy": policy,
});

// Each tab runs its own engine; the first one that knows the message answers.
async function body(request: string) {
	const clients = await self.clients.matchAll({
		type: "window",
		includeUncontrolled: true,
	});
	for (const client of clients) {
		if (client.frameType !== "top-level") continue;
		const { bytes, policy } = await ask(client, request);
		if (bytes) return new Response(bytes, { headers: headers(policy) });
	}
	return new Response(null, { status: 404 });
}

self.addEventListener("fetch", (event) => {
	const url = new URL(event.request.url);
	if (url.origin !== self.location.origin || !url.pathname.startsWith("/body/"))
		return;
	event.respondWith(
		body(decodeURIComponent(url.pathname.slice("/body/".length)) + url.search),
	);
});
