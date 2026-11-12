import type { ReactNode } from "react";
import { useId } from "react";
import { useDraft } from "../presentation";

// A message box with its controls under it, as in the reply panel and an agent's chat. The text
// is input before commit (useDraft); `children` are the controls, which send nothing until the
// contract has an intent for them.
export function Composer({
	label,
	placeholder,
	children,
}: {
	label: string;
	placeholder: string;
	children: ReactNode;
}) {
	const id = useId();
	const [text, setText] = useDraft();
	return (
		<div className="tile composer">
			<label htmlFor={id} className="sr-only">
				{label}
			</label>
			<textarea
				id={id}
				rows={2}
				placeholder={placeholder}
				value={text}
				onChange={(event) => setText(event.target.value)}
			/>
			<div className="composer-bar">{children}</div>
		</div>
	);
}
