import { build } from "esbuild";
import fs from "node:fs/promises";
import path from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const tmpDir = path.join(root, "tmp");
const outfile = path.join(tmpDir, "dashboard-smoke.mjs");

await fs.mkdir(tmpDir, { recursive: true });

await build({
  absWorkingDir: root,
  bundle: true,
  entryPoints: [path.join(root, "src", "ui", "dashboard-smoke.ts")],
  outfile,
  format: "esm",
  platform: "node",
  target: "es2022",
  sourcemap: false,
});

const smoke = await import(`${pathToFileURL(outfile).href}?t=${Date.now()}`);
const report = smoke.dashboardSectionSmokeReport();

console.log(`Friday dashboard UI smoke: ${report.scoreOutOf100}/100`);
for (const check of report.checks) {
  console.log(`- [${check.passed ? "passed" : "failed"}] ${check.id}: ${check.evidence}`);
}

if (report.scoreOutOf100 !== 100) {
  process.exitCode = 1;
}
