import { Icon } from "./Icon";

// The one empty state of a queue (Approvals, Quarantine): a check, what is clear, and what happens next.
export function AllClear({
	title,
	detail,
}: {
	title: string;
	detail?: string;
}) {
	return (
		<div className="all-clear settle">
			<Icon name="check" size="lg" className="ok" />
			<span className="strong">{title}</span>
			{detail && <span className="sm subtle">{detail}</span>}
		</div>
	);
}
