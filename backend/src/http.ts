import type { RpcTable } from "./rpc";

const PROTO = "application/proto";

const status = (code: number, headers?: HeadersInit) =>
	new Response(null, { status: code, headers });

export function handleRequest(table: RpcTable) {
	return async (req: Request): Promise<Response> => {
		const rpc = table.get(new URL(req.url).pathname);
		if (!rpc) return status(404);
		if (req.method !== "POST") return status(405, { Allow: "POST" });
		if (req.headers.get("content-type") !== PROTO) return status(415);

		let call: () => Promise<Uint8Array>;
		try {
			call = rpc(new Uint8Array(await req.arrayBuffer()));
		} catch {
			return status(400);
		}
		try {
			return new Response((await call()).slice(), {
				headers: { "Content-Type": PROTO },
			});
		} catch (error) {
			console.error(error);
			return status(500);
		}
	};
}

export function serve(table: RpcTable, port: number) {
	return Bun.serve({ port, fetch: handleRequest(table) });
}
