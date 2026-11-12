// Joins class names, skipping the ones a condition left out.
export const cx = (...names: (string | false | undefined)[]) =>
	names.filter(Boolean).join(" ");
