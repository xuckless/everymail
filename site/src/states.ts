/** Server-rendered attributes for a child shown in the listed states of a stack that starts in `initial`. */
export const when = (initial: string, states: string) => {
	const hidden = !states.split(" ").includes(initial);
	return {
		"data-s": states,
		class: hidden ? "hide" : undefined,
		"aria-hidden": hidden ? ("true" as const) : undefined,
	};
};

export const queueNote = (asks: string[]) =>
	asks.length
		? `Queued for approval: ${asks.join(", ")}`
		: "Nothing queued: agents act on their own";
