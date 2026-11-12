import type { CSSProperties, ReactNode } from "react";
import { type IntentKind, unset } from "../engine";
import type { NavDraft } from "../gen/ui/v1/compose_pb";
import { cx } from "../ui/cx";

// A saved draft in the sidebar and the phone's mailbox menu. What a press does follows what the
// Backend reports of its send (NavDraft.send): an unsent draft, or one it could not send, opens in
// the composer; one it holds (in the undo window or scheduled) is taken back with its action;
// one being sent does nothing.
export function DraftRow({
	draft,
	onPick,
	lead,
	className,
	style,
	role,
}: {
	draft: NavDraft;
	onPick: (intent: IntentKind) => void;
	lead?: ReactNode;
	className: string;
	style?: CSSProperties;
	role?: "menuitem";
}) {
	const { draftId } = draft;
	const open: IntentKind = { case: "openDraft", value: { draftId } };
	const press = ((): { intent: IntentKind | undefined; action: string } => {
		switch (draft.send.case) {
			case "unsent":
			case "notSent":
				return { intent: open, action: "" };
			case "held":
				return {
					intent: { case: "cancelSend", value: { draftId } },
					action: draft.send.value.action,
				};
			case "sending":
				return { intent: undefined, action: "" };
			case undefined:
				return unset("NavDraft.send");
		}
	})();
	return (
		<button
			type="button"
			role={role}
			className={cx(className, "row-in")}
			style={style}
			disabled={press.intent === undefined}
			onClick={() => press.intent && onPick(press.intent)}
		>
			{lead}
			<span className="grow two-line">
				<span className="trunc">{draft.title}</span>
				<span
					className={cx(
						"xs trunc",
						draft.send.case === "notSent" ? "bad" : "subtle",
					)}
				>
					{draft.detail}
				</span>
			</span>
			{press.action && <span className="xs link">{press.action}</span>}
		</button>
	);
}
