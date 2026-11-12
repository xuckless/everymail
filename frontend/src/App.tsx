import markUrl from "../../assets/mark.svg";
import type { ConnectionFailed, ShellView } from "./gen/ui/v1/shell_pb";

const labels = [
	"Connecting",
	"Connected",
	"Backend unreachable",
	"Database unavailable",
] as const;
type Label = (typeof labels)[number];

function failedLabel({ reason }: ConnectionFailed): Label {
	switch (reason.case) {
		case "backendUnreachable":
			return "Backend unreachable";
		case "databaseUnavailable":
			return "Database unavailable";
		case undefined:
			throw new Error("ConnectionFailed without a reason");
	}
}

type StatusProps = {
	dot: "run" | "done" | "err";
	current: Label;
	onRetry?: () => void;
};

function Status({ dot, current, onRetry }: StatusProps) {
	return (
		<p className="status" role="status">
			<span className={`dot ${dot}`} />
			<span className="labels">
				{labels.map((label) => (
					<span
						key={label}
						className={label === current ? undefined : "hide"}
						aria-hidden={label !== current}
					>
						{label}
					</span>
				))}
			</span>
			{onRetry && (
				<button className="btn press settle" type="button" onClick={onRetry}>
					Retry
				</button>
			)}
		</p>
	);
}

function Connection({
	view,
	onRetry,
}: {
	view: ShellView;
	onRetry: () => void;
}) {
	const state = view.connection?.state;
	switch (state?.case) {
		case "connecting":
			return <Status dot="run" current="Connecting" />;
		case "connected":
			return <Status dot="done" current="Connected" />;
		case "failed":
			return (
				<Status
					dot="err"
					current={failedLabel(state.value)}
					onRetry={onRetry}
				/>
			);
		case undefined:
			throw new Error("ShellView without a connection state");
	}
}

export function App({
	view,
	onRetry,
}: {
	view: ShellView;
	onRetry: () => void;
}) {
	return (
		<main className="shell">
			<img className="mark" src={markUrl} alt="" />
			<h1>EveryMail</h1>
			<Connection view={view} onRetry={onRetry} />
		</main>
	);
}
