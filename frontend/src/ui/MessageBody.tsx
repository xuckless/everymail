import { useEffect, useRef } from "react";
import { useEngine } from "../engine";

// A message body: the engine's HTML document at `url` (MessageExpanded.body_url,
// ReadingPreview.body_url) in a sandboxed iframe, so the body never enters the JavaScript heap.
// The engine's one script in the body posts its height as a bare number of CSS pixels on load and
// on every resize; the iframe takes that height. Every screen that shows a body uses this.
export function MessageBody({ url, title }: { url: string; title: string }) {
	const { bodySandbox } = useEngine();
	const frame = useRef<HTMLIFrameElement>(null);
	useEffect(() => {
		const size = (event: MessageEvent) => {
			const element = frame.current;
			if (
				element &&
				event.source === element.contentWindow &&
				typeof event.data === "number"
			)
				element.style.height = `${event.data}px`;
		};
		addEventListener("message", size);
		return () => removeEventListener("message", size);
	}, []);
	return (
		<iframe
			ref={frame}
			className="body-frame"
			src={url}
			title={title}
			sandbox={bodySandbox}
		/>
	);
}
