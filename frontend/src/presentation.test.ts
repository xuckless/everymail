import { expect, test } from "bun:test";
import {
	addressStep,
	type OverlayEvent,
	type OverlayPhase,
	overlayStep,
} from "./presentation";

const run = (events: OverlayEvent[], from: OverlayPhase = "closed") =>
	events.reduce(overlayStep, from);

test("an overlay opens, settles open, closes and settles closed", () => {
	expect(run(["toggle"])).toBe("opening");
	expect(run(["toggle", "settled"])).toBe("open");
	expect(run(["toggle", "settled", "dismiss"])).toBe("closing");
	expect(run(["toggle", "settled", "toggle", "settled"])).toBe("closed");
});

test("an overlay reverses mid-animation and ignores what does not apply", () => {
	expect(run(["toggle", "dismiss"])).toBe("closing");
	expect(run(["toggle"], "closing")).toBe("opening");
	expect(run(["dismiss", "settled"])).toBe("closed");
	expect(run(["settled"], "open")).toBe("open");
});

test("the address bar follows the engine's path", () => {
	expect(addressStep("/u/0/inbox", "/u/0/inbox", "other")).toBe("keep");
	expect(addressStep("/u/0/inbox", "/u/0/thread/t", "other")).toBe("push");
	// A loaded or popped path the engine rewrote replaces it, so Back never lands on it again.
	expect(addressStep("/", "/u/0/inbox", "open-path")).toBe("replace");
});
