import type { ReactNode } from "react";
import { modelSettings } from "../chrome/places";
import { unset, useSend } from "../engine";
import type { Model } from "../gen/ui/v1/common_pb";
import type { ModelChoice } from "../gen/ui/v1/thread_pb";
import { Check } from "./Check";
import { ModelChip } from "./Logo";
import { Popover, type TriggerProps } from "./Overlay";
import { stagger } from "./stagger";

// One row of a model menu: a ready model to pick, with its provider, or one whose provider has no
// key, which leads to the Models settings to add it.
export type ModelRow =
	| { kind: "ready"; model: Model; selected: boolean; provider: string }
	| { kind: "needsKey"; model: Model; label: string };

// The rows of an engine's model choices (the reply panel, an agent's settings).
export const choiceRows = (choices: ModelChoice[]) => choices.map(choiceRow);

function choiceRow(choice: ModelChoice): ModelRow {
	const model = choice.model ?? unset("ModelChoice.model");
	const { availability } = choice;
	switch (availability.case) {
		case "ready":
			return {
				kind: "ready",
				model,
				selected: choice.selected,
				provider: availability.value.providerName,
			};
		case "needsKey":
			return { kind: "needsKey", model, label: availability.value.label };
		case undefined:
			return unset("ModelChoice.availability");
	}
}

// The one model menu, for the reply panel, an agent's settings and the task models in Settings.
export function ModelMenu({
	label,
	rows,
	onPick,
	trigger,
	align,
	className,
}: {
	label: string;
	rows: ModelRow[];
	onPick: (modelName: string) => void;
	trigger: (props: TriggerProps) => ReactNode;
	align?: "start" | "end" | "stretch";
	className?: string;
}) {
	const send = useSend();
	return (
		<Popover
			label={label}
			align={align}
			className={className}
			trigger={trigger}
		>
			{(dismiss) =>
				rows.map((row, i) => (
					<Item
						key={row.model.name}
						row={row}
						index={i}
						pick={() => {
							dismiss();
							onPick(row.model.name);
						}}
						addKey={() => {
							dismiss();
							send(modelSettings);
						}}
					/>
				))
			}
		</Popover>
	);
}

function Item({
	row,
	index,
	pick,
	addKey,
}: {
	row: ModelRow;
	index: number;
	pick: () => void;
	addKey: () => void;
}) {
	switch (row.kind) {
		case "ready":
			return (
				<button
					type="button"
					role="menuitemradio"
					aria-checked={row.selected}
					className="menu-row press row-in"
					style={stagger(index)}
					onClick={pick}
				>
					<ModelChip model={row.model} />
					<span className="grow" />
					<span className="xs subtle">{row.provider}</span>
					<Check on={row.selected} />
				</button>
			);
		case "needsKey":
			return (
				<button
					type="button"
					role="menuitem"
					className="menu-row press row-in subtle"
					style={stagger(index)}
					onClick={addKey}
				>
					<ModelChip model={row.model} />
					<span className="grow" />
					<span className="xs">{row.label}</span>
				</button>
			);
	}
}
