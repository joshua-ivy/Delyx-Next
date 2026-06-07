import { existsSync, readdirSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join } from "node:path";

const repoRoot = fileURLToPath(new URL("../../..", import.meta.url));
const failures = [];
const exe = join(repoRoot, "target", "release", "delyx-next-desktop.exe");
const nsisDir = join(repoRoot, "target", "release", "bundle", "nsis");
const installer = installerPath();

check(existsSync(exe), "release executable must exist; run npm run package:windows first");
check(installer !== undefined, "NSIS setup executable must exist");
if (existsSync(exe)) {
  check(statSync(exe).size > 0, "release executable must not be empty");
}
if (installer) {
  check(statSync(installer).size > 0, "NSIS installer must not be empty");
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(`Tauri smoke passed: ${installer}`);

function installerPath() {
  if (!existsSync(nsisDir)) {
    return undefined;
  }
  const setup = readdirSync(nsisDir).find((entry) => entry.endsWith("_x64-setup.exe"));
  return setup ? join(nsisDir, setup) : undefined;
}

function check(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}
