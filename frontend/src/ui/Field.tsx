import { useId } from "react";
import { cx } from "./cx";

// A labelled text field (`.field`): one line, or `rows` lines as a textarea. Its value is the
// screen's: text the engine has no intent for yet stays in a useDraft of presentation.ts (input
// before commit). `hideLabel` keeps the label for screen readers only, where the design shows none.
export function Field({
	label,
	value,
	onChange,
	type = "text",
	rows,
	placeholder,
	hint,
	hideLabel,
	className,
}: {
	label: string;
	value: string;
	onChange: (value: string) => void;
	type?: "text" | "email" | "password";
	rows?: number;
	placeholder?: string;
	hint?: string;
	hideLabel?: boolean;
	className?: string;
}) {
	const id = useId();
	const common = {
		id,
		value,
		placeholder,
		onChange: (event: { target: { value: string } }) =>
			onChange(event.target.value),
	};
	return (
		<span className={cx("field", className)}>
			<label htmlFor={id} className={hideLabel ? "sr-only" : "label"}>
				{label}
			</label>
			{rows ? (
				<textarea rows={rows} {...common} />
			) : (
				<input type={type} {...common} />
			)}
			{hint && <span className="xs subtle">{hint}</span>}
		</span>
	);
}
