import { fromBinary } from "@bufbuild/protobuf";
import { LazyMotion, MotionConfig } from "motion/react";
import { useEffect, useLayoutEffect } from "react";
import { type Engine, EngineContext, unset, useEngine } from "./engine";
import { Density, Theme } from "./gen/ui/v1/common_pb";
import { ShellViewSchema } from "./gen/ui/v1/shell_pb";
import { RenderBoundary, showDensity, showTheme } from "./presentation";
import { Screen } from "./screens/Screen";
import { useTiming } from "./ui/motion";
import { OutcomesContext } from "./ui/Outcome";

// The one place a ShellView is decoded and rendered. A decode failure or an unset oneof throws
// while rendering and ends in the one error boundary, which shows a plain message.
export function App({ bytes, engine }: { bytes: Uint8Array; engine: Engine }) {
	return (
		<EngineContext value={engine}>
			<LazyMotion features={motionFeatures} strict>
				<MotionConfig reducedMotion="user">
					<RenderBoundary failed={(message) => <Failure message={message} />}>
						<Shell bytes={bytes} />
					</RenderBoundary>
				</MotionConfig>
			</LazyMotion>
		</EngineContext>
	);
}

const motionFeatures = () =>
	import("./ui/motion-features").then((module) => module.default);

function Shell({ bytes }: { bytes: Uint8Array }) {
	const view = fromBinary(ShellViewSchema, bytes);
	const { showPath } = useEngine();
	const { reveal } = useTiming();
	// The engine's theme and density on the document root (presentation.ts).
	// biome-ignore lint/correctness/useExhaustiveDependencies: a new theme is shown, not a new timing
	useLayoutEffect(() => showTheme(themeOf(view.theme), reveal), [view.theme]);
	useLayoutEffect(() => showDensity(densityOf(view.density)), [view.density]);
	// Every view: a popped path the engine rewrote to the path it already showed must be put back.
	useEffect(() => showPath(view.path, view.history), [showPath, view]);
	return (
		<OutcomesContext value={view.outcomes}>
			<Screen view={view} />
		</OutcomesContext>
	);
}

function themeOf(theme: Theme) {
	switch (theme) {
		case Theme.SYSTEM:
			return "system";
		case Theme.DARK:
			return "dark";
		case Theme.LIGHT:
			return "light";
		case Theme.UNSPECIFIED:
			return unset("Theme");
	}
}

function densityOf(density: Density) {
	switch (density) {
		case Density.COMPACT:
			return "compact";
		case Density.COMFORTABLE:
			return "comfortable";
		case Density.ROOMY:
			return "roomy";
		case Density.UNSPECIFIED:
			return unset("Density");
	}
}

function Failure({ message }: { message: string }) {
	return (
		<main className="failure" role="alert">
			<p className="h2">EveryMail stopped</p>
			<p className="sm muted">{message}</p>
		</main>
	);
}
