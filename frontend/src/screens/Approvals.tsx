import "./approvals.css";
import { match, unset } from "../engine";
import type { ApprovalItem, ApprovalsView } from "../gen/ui/v1/approvals_pb";
import { useLayout } from "../presentation";
import { AllClear } from "../ui/AllClear";
import {
	ApprovalActions,
	approvalOf,
	approvalTone,
} from "../ui/ApprovalActions";
import { Dot } from "../ui/Dot";
import { Page } from "../ui/Page";
import { Row, Rows } from "../ui/Rows";
import type { ScreenProps } from "./Screen";

// What agents left waiting for the user. Each item has room for what the agent wrote, why it asks
// and its actions (ApprovalActions). Approve fills and pulses when the engine reports it done; the item then leaves
// by its id and the rest close up. Edit opens the drafted reply in the composer.
export function Approvals({ view }: ScreenProps<ApprovalsView>) {
	const { queue } = view;
	return (
		<Page
			label="Approvals"
			title="Approvals"
			subtitle={match(queue, "ApprovalsView.queue", {
				waiting: (waiting) => waiting.count,
				clear: () => undefined,
			})}
			back
		>
			<Queue queue={queue} />
		</Page>
	);
}

function Queue({ queue }: { queue: ApprovalsView["queue"] }) {
	switch (queue.case) {
		case "waiting":
			return (
				<Rows label="Waiting for you" className="approvals">
					{queue.value.items.map((item, i) => (
						<Row key={item.id} index={i}>
							<Item item={item} />
						</Row>
					))}
				</Rows>
			);
		case "clear":
			return <AllClear title={queue.value.text} />;
		case undefined:
			return unset("ApprovalsView.queue");
	}
}

function Item({ item }: { item: ApprovalItem }) {
	return (
		<article className="tile approval">
			<span className="xs subtle status-line">
				<Dot tone={approvalTone(approvalOf(item.state))} />
				<span className="grow">{item.agent}</span>
				<span>{item.time}</span>
			</span>
			<h2 className="h2">{item.title}</h2>
			<p className="well approval-detail">{item.detail}</p>
			<span className="sm subtle">{item.reason}</span>
			<span className="approval-actions">
				<ApprovalActions
					approvalId={item.id}
					state={item.state}
					action={actionOf(item)}
					size={useLayout() === "phone" ? "lg" : undefined}
				/>
			</span>
		</article>
	);
}

function actionOf(item: ApprovalItem) {
	switch (item.action.case) {
		case "send":
		case "unsubscribe":
			return item.action.case;
		case undefined:
			return unset("ApprovalItem.action");
	}
}
