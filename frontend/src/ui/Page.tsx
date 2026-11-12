import type { ReactNode } from "react";
import { useSend } from "../engine";
import { useLayout } from "../presentation";
import { IconButton } from "./Button";
import { cx } from "./cx";

// A screen that is one page in one panel: its head (back on the phone where the screen has a back
// affordance, the title, a line under it, actions on the right, then optional tabs) over its body.
// The body scrolls unless `fill` says the content lays out its own scrolling (a chat).
export function Page({
	label,
	title,
	subtitle,
	back,
	actions,
	tabs,
	fill,
	className,
	children,
}: {
	label: string;
	title: ReactNode;
	subtitle?: ReactNode;
	back?: boolean;
	actions?: ReactNode;
	tabs?: ReactNode;
	fill?: boolean;
	className?: string;
	children: ReactNode;
}) {
	const send = useSend();
	const phone = useLayout() === "phone";
	return (
		<section
			className={cx(phone ? "page page-phone" : "panel page", className)}
			aria-label={label}
		>
			<header className={phone ? "phone-head" : "page-head"}>
				<div className="page-title">
					{back && phone && (
						<IconButton
							icon="chevron-left"
							label="Back"
							size="lg"
							touch
							onClick={() => send({ case: "back", value: {} })}
						/>
					)}
					<span className="grow page-name">
						<h1 className="h1 trunc">{title}</h1>
						{subtitle && <span className="sm muted">{subtitle}</span>}
					</span>
					{actions && <span className="page-actions">{actions}</span>}
				</div>
				{tabs}
			</header>
			{fill ? (
				<div className="page-fill">{children}</div>
			) : (
				<div className="scroll page-body">{children}</div>
			)}
		</section>
	);
}
