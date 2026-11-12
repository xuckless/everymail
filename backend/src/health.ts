import { create } from "@bufbuild/protobuf";
import type { SQL } from "bun";
import {
	type CheckResponse,
	CheckResponseSchema,
} from "./gen/api/v1/health_pb";

export async function check(sql: SQL): Promise<CheckResponse> {
	try {
		await sql`SELECT 1`;
		return create(CheckResponseSchema, {
			database: { case: "ready", value: {} },
		});
	} catch {
		return create(CheckResponseSchema, {
			database: { case: "unavailable", value: {} },
		});
	}
}
