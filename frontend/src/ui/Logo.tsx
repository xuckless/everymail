import logos from "@everymail/design/logos.svg?url";
import { unset } from "../engine";
import {
	CalendarProvider,
	MailProvider,
	ModelProvider,
	SignInProvider,
} from "../gen/ui/v1/common_pb";

// Every company mark and the app's own: the one place a provider enum becomes a logo.
export type Brand =
	| { kind: "mark" }
	| { kind: "mail"; provider: MailProvider }
	| { kind: "calendar"; provider: CalendarProvider }
	| { kind: "model"; provider: ModelProvider }
	| { kind: "signIn"; provider: SignInProvider };

function mail(provider: MailProvider) {
	switch (provider) {
		case MailProvider.GMAIL:
			return "gmail";
		case MailProvider.OUTLOOK:
			return "outlook";
		case MailProvider.ICLOUD:
			return "icloud";
		case MailProvider.YAHOO:
			return "yahoo";
		case MailProvider.FASTMAIL:
			return "fastmail";
		case MailProvider.OTHER_IMAP:
			return "imap";
		case MailProvider.UNSPECIFIED:
			return unset("MailProvider");
	}
}

function calendar(provider: CalendarProvider) {
	switch (provider) {
		case CalendarProvider.GOOGLE_CALENDAR:
			return "gcal";
		case CalendarProvider.OUTLOOK_CALENDAR:
			return "outlook";
		// As drawn: CalDAV calendars such as iCloud and Fastmail.
		case CalendarProvider.CALDAV:
			return "icloud";
		case CalendarProvider.UNSPECIFIED:
			return unset("CalendarProvider");
	}
}

function model(provider: ModelProvider) {
	switch (provider) {
		case ModelProvider.ANTHROPIC:
			return "anthropic";
		case ModelProvider.OPENAI:
			return "openai";
		case ModelProvider.GOOGLE_GEMINI:
			return "gemini";
		case ModelProvider.OPENROUTER:
			return "openrouter";
		case ModelProvider.UNSPECIFIED:
			return unset("ModelProvider");
	}
}

function signIn(provider: SignInProvider) {
	switch (provider) {
		case SignInProvider.GOOGLE:
			return "google";
		case SignInProvider.MICROSOFT:
			return "microsoft";
		case SignInProvider.UNSPECIFIED:
			return unset("SignInProvider");
	}
}

function logoId(brand: Brand) {
	switch (brand.kind) {
		case "mark":
			return "mark";
		case "mail":
			return mail(brand.provider);
		case "calendar":
			return calendar(brand.provider);
		case "model":
			return model(brand.provider);
		case "signIn":
			return signIn(brand.provider);
	}
}

// Decorative unless `label` names it, as on a row whose account is shown only by its logo.
export function Logo({
	brand,
	size = 18,
	label,
	className,
}: {
	brand: Brand;
	size?: number;
	label?: string;
	className?: string;
}) {
	return (
		<svg
			className={className}
			width={size}
			height={size}
			role={label ? "img" : undefined}
			aria-label={label}
			aria-hidden={label ? undefined : true}
			style={{ flex: "none" }}
		>
			<use href={`${logos}#${logoId(brand)}`} />
		</svg>
	);
}

// A logo on its own tile, as in account and provider lists: `size` is the logo, `box` the tile.
export function LogoTile({
	brand,
	size,
	box,
}: {
	brand: Brand;
	size: number;
	box: number;
}) {
	return (
		<span
			className="logo-tile"
			style={{ width: box, height: box, borderRadius: Math.round(box * 0.28) }}
		>
			<Logo brand={brand} size={size} />
		</span>
	);
}
