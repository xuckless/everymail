import { unset, useSend } from "../../engine";
import type {
	AgentsView,
	ApprovalItem,
	ApprovalQueue,
} from "../../gen/ui/v1/agents_pb";
import { Button } from "../../ui/Button";
import { Dot } from "../../ui/Dot";
import { approvalDone, FillButton } from "../../ui/FillButton";
import { Icon } from "../../ui/Icon";
import { Overlap } from "../../ui/Overlap";
import { Row, Rows } from "../../ui/Rows";

// What agents ask before acting. Approve, Decline, Unsubscribe and Keep have no intent yet and are
// drawn without one; Edit opens the thread the draft answers.

function Actions({ item }: { item: ApprovalItem }) {
	const send = useSend();
	const done = approvalDone(item.state);
	const verb = (label: string) => (
		<FillButton
			done={done}
			label={label}
			doneLabel="Done"
			className="queue-main"
		/>
	);
	switch (item.action.case) {
		case "send": {
			const threadId = item.action.value.threadId;
			return (
				<>
					{verb("Approve and send")}
					<Button
						variant="soft"
						onClick={() => send({ case: "openThread", value: { threadId } })}
					>
						Edit
					</Button>
					<Button variant="ghost">Decline</Button>
				</>
			);
		}
		case "unsubscribe":
			return (
				<>
					{verb("Unsubscribe")}
					<Button variant="ghost">Keep</Button>
				</>
			);
		case undefined:
			return unset("ApprovalItem.action");
	}
}

// The queue item's element, which a chat turn waiting for it brings into view on wide layouts.
export const approvalElement = (approvalId: string) => `approval-${approvalId}`;

function Item({ item }: { item: ApprovalItem }) {
	return (
		<div
			className="tile queue-item"
			id={approvalElement(item.id)}
			tabIndex={-1}
		>
			<span className="xs subtle queue-meta">
				<Dot tone="wait" />
				<span className="grow">{item.agent}</span>
				<span>{item.time}</span>
			</span>
			<span className="strong">{item.title}</span>
			<span className="sm muted queue-detail">{item.detail}</span>
			<span className="xs subtle">{item.reason}</span>
			<span className="queue-actions">
				<Actions item={item} />
			</span>
		</div>
	);
}

function Waiting({ queue }: { queue: ApprovalQueue }) {
	return (
		<Rows label="Waiting for you" className="queue-rows">
			{queue.items.map((item, i) => (
				<Row key={item.id} index={i}>
					<Item item={item} />
				</Row>
			))}
		</Rows>
	);
}

function Clear() {
	return (
		<div className="settle queue-clear">
			<Icon name="check" size="lg" className="ok" />
			<span className="strong">All clear</span>
			<span className="sm subtle">
				Agents keep running. New items land here when an action is set to Ask
			</span>
		</div>
	);
}

function QueueBody({ queue }: { queue: AgentsView["queue"] }) {
	switch (queue.case) {
		case "waiting":
			return <Waiting queue={queue.value} />;
		case "clear":
			return <Clear />;
		case undefined:
			return unset("AgentsView.queue");
	}
}

function count(queue: AgentsView["queue"]) {
	switch (queue.case) {
		case "waiting":
			return queue.value.count;
		case "clear":
			return "";
		case undefined:
			return unset("AgentsView.queue");
	}
}

// The approval queue as a panel of its own (wide layouts) or a pane (phone).
export function QueuePanel({ queue }: { queue: AgentsView["queue"] }) {
	return (
		<aside className="panel queue" aria-label="Approval queue">
			<div className="queue-head">
				<h2 className="h2 grow">Waiting for you</h2>
				<Overlap id={count(queue)} className="sm subtle queue-count">
					{count(queue)}
				</Overlap>
			</div>
			<div className="scroll queue-scroll">
				<QueueBody queue={queue} />
			</div>
		</aside>
	);
}

export function QueuePane({ queue }: { queue: AgentsView["queue"] }) {
	return (
		<div className="scroll noscroll queue-pane">
			<QueueBody queue={queue} />
		</div>
	);
}
