import { expect, test } from "bun:test";
import { create, fromBinary, toBinary } from "@bufbuild/protobuf";
import {
	CheckRequestSchema,
	CheckResponseSchema,
	HealthService,
} from "./gen/api/v1/health_pb";
import { handleRequest } from "./http";
import { rpc } from "./rpc";

const path = "/api.v1.HealthService/Check";
const handle = handleRequest(
	new Map([
		rpc(HealthService.method.check, async () =>
			create(CheckResponseSchema, {
				database: { case: "unavailable", value: {} },
			}),
		),
	]),
);
const post = (body: BodyInit, contentType = "application/proto") =>
	handle(
		new Request(`http://x${path}`, {
			method: "POST",
			headers: { "Content-Type": contentType },
			body,
		}),
	);

test("round trips protobuf through the handler", async () => {
	const res = await post(
		toBinary(CheckRequestSchema, create(CheckRequestSchema)),
	);
	expect(res.status).toBe(200);
	expect(res.headers.get("content-type")).toBe("application/proto");
	const body = fromBinary(
		CheckResponseSchema,
		new Uint8Array(await res.arrayBuffer()),
	);
	expect(body.database.case).toBe("unavailable");
});

test("unknown path is 404", async () => {
	const res = await handle(new Request("http://x/nope", { method: "POST" }));
	expect(res.status).toBe(404);
});

test("non-POST is 405 with Allow", async () => {
	const res = await handle(new Request(`http://x${path}`));
	expect(res.status).toBe(405);
	expect(res.headers.get("allow")).toBe("POST");
});

test("wrong content type is 415", async () => {
	expect((await post(new Uint8Array(), "application/json")).status).toBe(415);
});

test("undecodable body is 400", async () => {
	expect((await post(new Uint8Array([0xff]))).status).toBe(400);
});
