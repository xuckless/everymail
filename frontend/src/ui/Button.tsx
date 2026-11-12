import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cx } from "./cx";
import { Icon, type IconName } from "./Icon";

type Native = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "className">;

// A `.btn` that scales to 0.97 while pressed (`.press`). A control whose intent the contract does
// not define yet is rendered without onClick.
export function Button({
	variant,
	size,
	className,
	type = "button",
	children,
	...native
}: Native & {
	variant: "primary" | "soft" | "ghost";
	size?: "lg";
	className?: string;
	children: ReactNode;
}) {
	return (
		<button
			type={type}
			className={cx("btn", variant, size, "press", className)}
			{...native}
		>
			{children}
		</button>
	);
}

// An icon-only button; `label` is its accessible name. `touch` is the 44px phone target.
export function IconButton({
	icon,
	label,
	touch,
	size,
	className,
	type = "button",
	...native
}: Native & {
	icon: IconName;
	label: string;
	touch?: boolean;
	size?: "sm" | "lg";
	className?: string;
}) {
	return (
		<button
			type={type}
			className={cx("icon-btn", touch && "touch", "press", className)}
			aria-label={label}
			{...native}
		>
			<Icon name={icon} size={size} />
		</button>
	);
}
