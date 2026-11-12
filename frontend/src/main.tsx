import "@fontsource-variable/geist";
import "virtual:tokens.css";
import "./app.css";
import { create, fromBinary, toBinary } from "@bufbuild/protobuf";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { connectEngine } from "./engine";
import { IntentSchema, ShellViewSchema } from "./gen/ui/v1/shell_pb";

const retryConnection = toBinary(
	IntentSchema,
	create(IntentSchema, { kind: { case: "retryConnection", value: {} } }),
);

const root = createRoot(
	document.body.appendChild(document.createElement("div")),
);
const engine = connectEngine((bytes) =>
	root.render(
		<App
			view={fromBinary(ShellViewSchema, bytes)}
			onRetry={() => engine.send(retryConnection)}
		/>,
	),
);
