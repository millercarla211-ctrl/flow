import type { IncomingMessage, ServerResponse } from "node:http";

import { toNodeHandler } from "better-auth/node";
import { getAuth } from "../../src/server/auth";

let authHandler: ReturnType<typeof toNodeHandler> | null = null;

export default function handler(request: IncomingMessage, response: ServerResponse) {
  authHandler ??= toNodeHandler(getAuth());
  return authHandler(request, response);
}
