import {
	type DescMessage,
	type DescMethodUnary,
	fromBinary,
	type MessageShape,
	toBinary,
} from "@bufbuild/protobuf";

/** Decodes a request body, or throws if it is not a valid message. Returns the call to run. */
export type Rpc = (body: Uint8Array) => () => Promise<Uint8Array>;

export type RpcTable = ReadonlyMap<string, Rpc>;

export function rpc<I extends DescMessage, O extends DescMessage>(
	method: DescMethodUnary<I, O>,
	handler: (request: MessageShape<I>) => Promise<MessageShape<O>>,
): [path: string, rpc: Rpc] {
	return [
		`/${method.parent.typeName}/${method.name}`,
		(body) => {
			const request = fromBinary(method.input, body);
			return async () => toBinary(method.output, await handler(request));
		},
	];
}
