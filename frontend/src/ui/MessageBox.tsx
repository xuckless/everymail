import { useId } from "react";
import { useDraft } from "../presentation";
import { Button } from "./Button";

// A message box with its send button under it, as in the reply panel and an agent's chat. The text
// is input before commit (useDraft); the button and ⌘Enter or Ctrl+Enter send it. The engine says
// when it took the text with a new revision: the caller keys the box by it, so it starts empty.
export function MessageBox({
	label,
	placeholder,
	action,
	onSubmit,
}: {
	label: string;
	placeholder: string;
	action: string;
	onSubmit: (text: string) => void;
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
				onKeyDown={(event) => {
					if (event.key === "Enter" && (event.metaKey || event.ctrlKey))
						onSubmit(text);
				}}
			/>
			<div className="composer-bar">
				<span className="grow" />
				<Button variant="primary" onClick={() => onSubmit(text)}>
					{action}
				</Button>
			</div>
		</div>
	);
}
