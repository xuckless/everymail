import "./settings.css";
import { unset, useSend } from "../engine";
import type {
	SettingsSection,
	SettingsTab,
	SettingsView,
} from "../gen/ui/v1/settings_pb";
import { useLayout } from "../presentation";
import { cx } from "../ui/cx";
import { Icon } from "../ui/Icon";
import { Overlap } from "../ui/Overlap";
import { Popover } from "../ui/Overlay";
import { SelectionMark, useSelectionGroup } from "../ui/Selection";
import type { ScreenProps } from "./Screen";
import {
	Accounts,
	AgentPermissions,
	Appearance,
	BlockList,
	Models,
	Rules,
	YourData,
} from "./settings/sections";

// Settings: the section list and the open section. The open section is the engine's
// (open_settings); each section shows what its view gives.
// Desktop: the section list as a column beside the section. Phone: one column, the section list
// collapsed into a menu on top.
export function Settings({ view }: ScreenProps<SettingsView>) {
	const phone = useLayout() === "phone";
	return (
		<div className={cx("settings", phone && "settings-phone")}>
			{phone ? (
				<SectionMenu tabs={view.sections} />
			) : (
				<SectionNav tabs={view.sections} />
			)}
			<section className="panel settings-main">
				<Overlap
					id={view.section.case ?? unset("SettingsView.section")}
					className="settings-stack"
				>
					<div className="scroll settings-scroll">
						<SectionBody view={view} />
					</div>
				</Overlap>
			</section>
		</div>
	);
}

function SectionBody({ view }: { view: SettingsView }) {
	const section = view.section;
	switch (section.case) {
		case "accounts":
			return <Accounts section={section.value} />;
		case "agentPermissions":
			return <AgentPermissions section={section.value} />;
		case "rules":
			return <Rules section={section.value} />;
		case "blockList":
			return <BlockList section={section.value} />;
		case "models":
			return <Models section={section.value} />;
		case "appearance":
			return <Appearance section={section.value} />;
		case "yourData":
			return <YourData section={section.value} />;
		case undefined:
			return unset("SettingsView.section");
	}
}

function useOpenSection() {
	const send = useSend();
	return (section: SettingsSection) => () =>
		send({ case: "openSettings", value: { section } });
}

function SectionNav({ tabs }: { tabs: SettingsTab[] }) {
	const open = useOpenSection();
	const group = useSelectionGroup();
	return (
		<nav className="panel settings-nav" aria-label="Settings sections">
			<h2 className="h2 settings-nav-title">Settings</h2>
			<div>
				{tabs.map((tab) => (
					<button
						key={tab.section}
						type="button"
						className={cx(
							"press selectable settings-nav-item",
							tab.selected && "on",
						)}
						aria-current={tab.selected ? "page" : undefined}
						onClick={open(tab.section)}
					>
						{tab.selected && <SelectionMark group={group} />}
						{tab.label}
					</button>
				))}
			</div>
		</nav>
	);
}

function SectionMenu({ tabs }: { tabs: SettingsTab[] }) {
	const open = useOpenSection();
	return (
		<nav className="settings-menu" aria-label="Settings sections">
			<Popover
				label="Settings sections"
				align="stretch"
				trigger={(props) => (
					<button
						type="button"
						className="press settings-menu-trigger"
						{...props}
					>
						<span className="h2 grow">Settings</span>
						<Icon name="chevron-down" />
					</button>
				)}
			>
				{(dismiss) =>
					tabs.map((tab) => (
						<button
							key={tab.section}
							type="button"
							role="menuitemradio"
							aria-checked={tab.selected}
							className={cx("menu-row touch press", tab.selected && "on")}
							onClick={() => {
								open(tab.section)();
								dismiss();
							}}
						>
							{tab.label}
						</button>
					))
				}
			</Popover>
		</nav>
	);
}
