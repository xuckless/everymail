import type { CSSProperties } from "react";

// A list row's entrance delay: 30ms (t-stagger) per row before it (design/README.md). It is the
// delay itself, so any entrance class (`settle`, `row-in`, `reveal`) staggers without a second class.
export const stagger = (index: number): CSSProperties => ({
	animationDelay: `calc(${index} * var(--t-stagger))`,
});
