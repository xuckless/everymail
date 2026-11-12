import { unset, useSend } from "../engine";
import type { ConnectionFailed, ConnectionView } from "../gen/ui/v1/shell_pb";
import { IconButton } from "../ui/Button";
import { cx } from "../ui/cx";
import { Dot, type Tone } from "../ui/Dot";

// The connection to the Backend, small and out of the way: a line in the sidebar, a dot elsewhere.
// Failed offers Retry. The line's labels overlap old and new states (`.stack`, `.hide`).
const labels = [
	"Connecting",
	"Connected",
	"Backend unreachable",
	"Database unavailable",
] as const;
type Label = (typeof labels)[number];

type Status = { label: Label; tone: Tone; failed: boolean };

function status(connection: ConnectionView | undefined): Status {
	const state = connection?.state;
	switch (state?.case) {
		case "connecting":
			return { label: "Connecting", tone: "run", failed: false };
		case "connected":
			return { label: "Connected", tone: "done", failed: false };
		case "failed":
			return failure(state.value);
		case undefined:
			return unset("ConnectionView.state");
	}
}

function failure({ reason }: ConnectionFailed): Status {
	switch (reason.case) {
		case "backendUnreachable":
			return { label: "Backend unreachable", tone: "err", failed: true };
		case "databaseUnavailable":
			return { label: "Database unavailable", tone: "err", failed: true };
		case undefined:
			return unset("ConnectionFailed.reason");
	}
}

export function ConnectionStatus({
	connection,
	form,
	className,
}: {
	connection: ConnectionView | undefined;
	form: "line" | "dot";
	className?: string;
}) {
	const send = useSend();
	const { label, tone, failed } = status(connection);
	const retry = () => send({ case: "retryConnection", value: {} });
	switch (form) {
		case "line":
			return (
				<p className={cx("connection", className)} role="status">
					<Dot tone={tone} />
					<span className="stack xs subtle">
						{labels.map((each) => (
							<span
								key={each}
								className={cx("trunc", each !== label && "hide")}
								aria-hidden={each !== label}
							>
								{each}
							</span>
						))}
					</span>
					{failed && (
						<button type="button" className="link xs settle" onClick={retry}>
							Retry
						</button>
					)}
				</p>
			);
		case "dot":
			return (
				<span className={cx("connection", className)} role="status">
					<Dot tone={tone} label={label} />
					{failed && (
						<IconButton
							icon="retry"
							size="sm"
							label={`${label}. Retry`}
							className="settle"
							onClick={retry}
						/>
					)}
				</span>
			);
	}
}
