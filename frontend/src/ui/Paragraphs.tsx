import { cx } from "./cx";
import { stagger } from "./stagger";

// Engine-written paragraphs, such as an agent's draft. With `settle` they settle in one after
// another with the 30ms stagger, as details do when they arrive.
export function Paragraphs({
	texts,
	settle,
	className,
}: {
	texts: string[];
	settle?: boolean;
	className?: string;
}) {
	return (
		<div className={cx("paragraphs", className)}>
			{texts.map((text, i) => (
				<p
					// biome-ignore lint/suspicious/noArrayIndexKey: paragraphs carry no id and never reorder
					key={i}
					className={cx(settle && "settle stagger")}
					style={settle ? stagger(i) : undefined}
				>
					{text}
				</p>
			))}
		</div>
	);
}
