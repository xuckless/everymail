import { unset, useSend } from "../engine";
import type { Reference as ReferenceView } from "../gen/ui/v1/common_pb";
import { cx } from "./cx";

// What an agent's result, run or answer is about, as a plain link: a thread opens, an event opens
// on the calendar, an approval opens Approvals, quarantined mail opens on Quarantine.
export function Reference({
	reference,
	className,
}: {
	reference: ReferenceView;
	className?: string;
}) {
	const send = useSend();
	const { target } = reference;
	return (
		<button
			type="button"
			className={cx("link reference", className)}
			onClick={() => {
				switch (target.case) {
					case "thread":
						return send({
							case: "openThread",
							value: { threadId: target.value.threadId },
						});
					case "event":
						return send({
							case: "selectEvent",
							value: { eventId: target.value.eventId },
						});
					case "approval":
						return send({ case: "openApprovals", value: {} });
					case "quarantined":
						return send({
							case: "selectQuarantined",
							value: { quarantinedId: target.value.quarantinedId },
						});
					case undefined:
						return unset("Reference.target");
				}
			}}
		>
			{reference.label}
		</button>
	);
}

// Several references, one per line.
export function References({
	references,
	className,
}: {
	references: ReferenceView[];
	className?: string;
}) {
	if (references.length === 0) return null;
	return (
		<span className={cx("references", className)}>
			{references.map((reference) => (
				<Reference key={reference.label} reference={reference} />
			))}
		</span>
	);
}
