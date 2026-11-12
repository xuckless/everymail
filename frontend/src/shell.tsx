import { fromBinary } from "@bufbuild/protobuf";
import { LazyMotion, MotionConfig } from "motion/react";
import { useEffect, useLayoutEffect } from "react";
import { type Engine, EngineContext, unset, useEngine } from "./engine";
import { Density, Theme } from "./gen/ui/v1/common_pb";
import { ShellViewSchema } from "./gen/ui/v1/shell_pb";
import { RenderBoundary } from "./presentation";
import { Screen } from "./screens/Screen";

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
	useLayoutEffect(() => applyTheme(view.theme), [view.theme]);
	useLayoutEffect(() => applyDensity(view.density), [view.density]);
	useEffect(() => showPath(view.path), [showPath, view.path]);
	return <Screen view={view} />;
}

// The engine's theme on the document root (design/README.md): an attribute for Dark and Light,
// none for System. A change is revealed with the design's circular mask where View Transitions
// exist; a control may set --vx and --vy on the root to start the circle where it was pressed.
// Elsewhere the colours crossfade (`theme-fade`) while the body's colours transition.
function applyTheme(theme: Theme) {
	const root = document.documentElement;
	const next = themeAttribute(theme);
	if (root.dataset.theme === next) return;
	const apply = () => {
		if (next) root.dataset.theme = next;
		else delete root.dataset.theme;
	};
	if (document.startViewTransition) {
		document.startViewTransition(apply);
		return;
	}
	root.classList.add("theme-fade");
	apply();
	// Once the body's colour transition has run (or at once when the colours did not change).
	requestAnimationFrame(() => {
		Promise.allSettled(
			document.body.getAnimations().map((animation) => animation.finished),
		).then(() => root.classList.remove("theme-fade"));
	});
}

function themeAttribute(theme: Theme) {
	switch (theme) {
		case Theme.SYSTEM:
			return undefined;
		case Theme.DARK:
			return "dark";
		case Theme.LIGHT:
			return "light";
		case Theme.UNSPECIFIED:
			return unset("Theme");
	}
}

// The engine's density on the document root, as the design package reads it.
function applyDensity(density: Density) {
	document.documentElement.dataset.density = densityAttribute(density);
}

function densityAttribute(density: Density) {
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
