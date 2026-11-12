export const must = <T extends Element>(
	selector: string,
	root: ParentNode = document,
): T => {
	const found = root.querySelector<T>(selector);
	if (!found) throw new Error(`missing ${selector}`);
	return found;
};

/** Makes `state` the root's state: children whose data-s lists it stay, the others fade out. */
export const show = (root: HTMLElement, state: string) => {
	root.dataset.state = state;
	for (const el of root.querySelectorAll<HTMLElement>("[data-s]")) {
		const hidden = !el.matches(`[data-s~="${state}"]`);
		el.classList.toggle("hide", hidden);
		el.ariaHidden = hidden ? "true" : null;
	}
};
