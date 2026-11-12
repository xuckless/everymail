import "./connect.css";
import { AnimatePresence } from "motion/react";
import { allInboxes } from "../chrome/places";
import { unset, useSend } from "../engine";
import type { MailProvider } from "../gen/ui/v1/common_pb";
import type {
	ConnectedAccount,
	ConnectImapForm,
	ConnectProvider,
	ConnectSigningIn,
	ConnectSyncing,
	ConnectView,
} from "../gen/ui/v1/connect_pb";
import { useDraft, useLayout } from "../presentation";
import { Button } from "../ui/Button";
import { accountStatus, Dot } from "../ui/Dot";
import { Field } from "../ui/Field";
import { Logo, LogoTile } from "../ui/Logo";
import { Overlap } from "../ui/Overlap";
import { PresentSheet } from "../ui/Overlay";
import { Row, Rows } from "../ui/Rows";
import { stagger } from "../ui/stagger";

// Bring your inboxes: the connected accounts, then one step of connecting another. Desktop shows
// the step in place of the provider grid, old and new overlapping; the phone lists providers and
// shows the other steps in a bottom sheet the engine opens and closes.
export function Connect({ view }: { view: ConnectView }) {
	return useLayout() === "phone" ? (
		<PhoneConnect view={view} />
	) : (
		<DesktopConnect view={view} />
	);
}

const mail = (provider: MailProvider) => ({ kind: "mail" as const, provider });

function DesktopConnect({ view }: { view: ConnectView }) {
	const step = view.step;
	return (
		<div className="connect scroll">
			<div className="connect-body">
				<header className="connect-head">
					<Logo brand={{ kind: "mark" }} size={48} />
					<h1 className="h1 connect-title">Bring your inboxes</h1>
					<p className="muted connect-lede">
						Add as many accounts as you like. Mail stays on your provider;
						EveryMail syncs and keeps your agents running
					</p>
				</header>
				{view.connected.length > 0 && (
					<section className="connect-group">
						<span className="label connect-label">Connected</span>
						<Connected accounts={view.connected} status="line" />
					</section>
				)}
				<Overlap id={step.case ?? unset("ConnectView.step")}>
					{step.case === "picking" ? (
						<ProviderGrid providers={view.providers} />
					) : (
						<div className="tile connect-step">
							<StepBody step={step} />
						</div>
					)}
				</Overlap>
				<footer className="connect-foot">
					<span className="sm subtle grow">
						Custom domains and hosted addresses on everyai.dev arrive later
					</span>
					<OpenInbox />
				</footer>
			</div>
		</div>
	);
}

function OpenInbox() {
	const send = useSend();
	return (
		<Button variant="primary" size="lg" onClick={() => send(allInboxes)}>
			Open inbox
		</Button>
	);
}

function PhoneConnect({ view }: { view: ConnectView }) {
	const send = useSend();
	const step = view.step;
	return (
		<div className="connect-phone">
			<div className="connect-phone-body scroll noscroll">
				<header className="connect-head">
					<Logo brand={{ kind: "mark" }} size={44} />
					<h1 className="h1">Your accounts</h1>
					<p className="muted connect-lede">
						Add every inbox. Agents can work across all of them
					</p>
				</header>
				<Connected accounts={view.connected} status="dot" />
				<section className="connect-group">
					<span className="label connect-label">Add an account</span>
					<ProviderList providers={view.providers} />
				</section>
			</div>
			<div className="connect-phone-foot">
				<OpenInbox />
			</div>
			<AnimatePresence>
				{step.case !== "picking" && (
					<PresentSheet
						key="step"
						label="Connect an account"
						onDismiss={() => send({ case: "back", value: {} })}
					>
						<Overlap id={step.case ?? unset("ConnectView.step")}>
							<div className="connect-step">
								<StepBody step={step} />
							</div>
						</Overlap>
					</PresentSheet>
				)}
			</AnimatePresence>
		</div>
	);
}

// Desktop prints each account's status; the phone shows only its dot, labelled.
function Connected({
	accounts,
	status,
}: {
	accounts: ConnectedAccount[];
	status: "line" | "dot";
}) {
	return (
		<Rows label="Connected accounts" className="connected">
			{accounts.map((account, i) => {
				const { tone, label } = accountStatus(account.status);
				return (
					<Row key={account.id} index={i} className="tile connected-row">
						<LogoTile brand={mail(account.provider)} size={20} box={32} />
						<span className="grow two-line">
							<span className="strong">{account.name}</span>
							<span className="xs subtle">{account.providerName}</span>
						</span>
						{status === "line" ? (
							<span className="sm muted status-line">
								<Dot tone={tone} />
								{label}
							</span>
						) : (
							<Dot tone={tone} label={label} />
						)}
					</Row>
				);
			})}
		</Rows>
	);
}

function ProviderGrid({ providers }: { providers: ConnectProvider[] }) {
	const send = useSend();
	return (
		<section className="connect-group">
			<span className="label connect-label">Add an account</span>
			<div className="connect-grid">
				{providers.map((each, i) => (
					<button
						key={each.provider}
						type="button"
						className="tile press row-in stagger provider-tile"
						style={stagger(i)}
						onClick={() =>
							send({
								case: "pickMailProvider",
								value: { provider: each.provider },
							})
						}
					>
						<LogoTile brand={mail(each.provider)} size={22} box={36} />
						<span className="two-line">
							<span className="strong">{each.name}</span>
							<span className="xs subtle">{each.method}</span>
						</span>
					</button>
				))}
			</div>
		</section>
	);
}

function ProviderList({ providers }: { providers: ConnectProvider[] }) {
	const send = useSend();
	return (
		<div className="tile provider-list">
			{providers.map((each, i) => (
				<button
					key={each.provider}
					type="button"
					className="menu-row touch press row-in stagger"
					style={stagger(i)}
					onClick={() =>
						send({
							case: "pickMailProvider",
							value: { provider: each.provider },
						})
					}
				>
					<LogoTile brand={mail(each.provider)} size={20} box={30} />
					<span className="grow">{each.name}</span>
					<span className="xs subtle">{each.method}</span>
				</button>
			))}
		</div>
	);
}

// The steps after picking, shared by the desktop tile and the phone sheet.
function StepBody({ step }: { step: ConnectView["step"] }) {
	switch (step.case) {
		case "signingIn":
			return <SigningIn step={step.value} />;
		case "imapForm":
			return <ImapForm step={step.value} />;
		case "syncing":
			return <Syncing step={step.value} />;
		case "picking":
			return null;
		case undefined:
			return unset("ConnectView.step");
	}
}

function SigningIn({ step }: { step: ConnectSigningIn }) {
	const send = useSend();
	return (
		<div className="connect-centered">
			<LogoTile brand={mail(step.provider)} size={30} box={52} />
			<span className="h2">{step.title}</span>
			<span className="sm muted connect-note">{step.note}</span>
			<span className="shimmer sm">{step.waiting}</span>
			<span className="connect-actions">
				<Button
					variant="soft"
					onClick={() => send({ case: "back", value: {} })}
				>
					Back
				</Button>
				<Button
					variant="primary"
					onClick={() => send({ case: "confirmConnect", value: {} })}
				>
					I’ve signed in
				</Button>
			</span>
		</div>
	);
}

function ImapForm({ step }: { step: ConnectImapForm }) {
	const send = useSend();
	const [address, setAddress] = useDraft();
	const [password, setPassword] = useDraft();
	return (
		<form
			className="connect-form"
			onSubmit={(event) => {
				event.preventDefault();
				send({ case: "confirmConnect", value: {} });
			}}
		>
			<span className="connect-step-head">
				<LogoTile brand={mail(step.provider)} size={24} box={40} />
				<span className="grow two-line">
					<span className="h2">{step.title}</span>
					<span className="sm subtle">
						Over IMAP with an app password, not your main password
					</span>
				</span>
			</span>
			<Field
				label="Email address"
				type="email"
				placeholder="you@example.com"
				value={address}
				onChange={setAddress}
			/>
			<Field
				label="App password"
				type="password"
				placeholder="xxxx-xxxx-xxxx-xxxx"
				value={password}
				onChange={setPassword}
				hint={step.passwordHint}
			/>
			<Server server={step.server} />
			<span className="connect-actions end">
				<Button
					variant="soft"
					onClick={() => send({ case: "back", value: {} })}
				>
					Back
				</Button>
				<Button variant="primary" type="submit">
					Connect
				</Button>
			</span>
		</form>
	);
}

function Server({ server }: { server: ConnectImapForm["server"] }) {
	switch (server.case) {
		case "known":
			return null;
		case "entry":
			return <ServerEntry defaultPort={server.value.defaultPort} />;
		case undefined:
			return unset("ConnectImapForm.server");
	}
}

function ServerEntry({ defaultPort }: { defaultPort: string }) {
	const [host, setHost] = useDraft();
	const [port, setPort] = useDraft(defaultPort);
	return (
		<span className="reveal connect-server">
			<Field
				label="IMAP server"
				placeholder="imap.example.com"
				value={host}
				onChange={setHost}
			/>
			<Field label="Port" value={port} onChange={setPort} />
		</span>
	);
}

function Syncing({ step }: { step: ConnectSyncing }) {
	return (
		<div className="connect-form">
			<span className="connect-step-head">
				<LogoTile brand={mail(step.provider)} size={24} box={40} />
				<span className="grow two-line">
					<span className="h2">{step.title}</span>
					<span className="sm subtle">
						Newest mail first, so you can start right away
					</span>
				</span>
			</span>
			<div
				className="bar"
				role="progressbar"
				aria-label={step.title}
				aria-valuenow={step.percent}
			>
				<i style={{ width: `${step.percent}%` }} />
			</div>
			<div className="connect-stats">
				{step.stats.map((stat, i) => (
					<span key={stat.label} className="settle stagger" style={stagger(i)}>
						<span className="connect-stat">{stat.value}</span>
						<span className="xs subtle">{stat.label}</span>
					</span>
				))}
			</div>
		</div>
	);
}
