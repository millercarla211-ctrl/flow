import { toNodeHandler } from "better-auth/node";
import { auth } from "../../src/server/auth";

export default toNodeHandler(auth);
