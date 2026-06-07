import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join } from "node:path";

const desktopRoot = fileURLToPath(new URL("..", import.meta.url));
const repoRoot = fileURLToPath(new URL("../../..", import.meta.url));
const failures = [];

const desktopPackage = readJson(join(desktopRoot, "package.json"));
const rootPackage = readJson(join(repoRoot, "package.json"));
const tauriConfig = readJson(join(desktopRoot, "src-tauri", "tauri.conf.json"));
const updateMetadata = readJson(join(desktopRoot, "release", "update-metadata.placeholder.json"));
const cargoToml = readFileSync(join(desktopRoot, "src-tauri", "Cargo.toml"), "utf8");
const tauriMain = readFileSync(join(desktopRoot, "src-tauri", "src", "main.rs"), "utf8");

check(rootPackage.scripts?.["package:windows"]?.includes("@delyx/desktop"), "root package:windows script must delegate to desktop workspace");
check(rootPackage.scripts?.["release:smoke"]?.includes("@delyx/desktop"), "root release:smoke script must delegate to desktop workspace");
check(desktopPackage.scripts?.["package:windows"]?.includes("--bundles nsis"), "desktop package:windows must build NSIS");
check(desktopPackage.scripts?.["release:smoke"]?.includes("release-smoke.mjs"), "desktop release:smoke script must run this smoke file");
check(tauriConfig.productName === "Delyx Next", "Tauri productName must be Delyx Next");
check(tauriConfig.identifier === "com.geaux.delyxnext", "Tauri identifier must stay separate from old Delyx");
check(tauriConfig.build?.frontendDist === "../dist", "Tauri build must package local Vite dist");
check(tauriConfig.build?.beforeBuildCommand === "..\\..\\.tools\\npm.cmd run build", "Tauri build must run the local frontend build");
check(tauriConfig.bundle?.active === true, "Tauri bundling must be active for release builds");
check(bundleTargets(tauriConfig).includes("nsis"), "Windows dev installer target must include nsis");
check(tauriConfig.bundle?.windows?.certificateThumbprint == null, "dev signing certificate must be absent");
check(tauriConfig.bundle?.windows?.digestAlgorithm == null, "dev signing digest must be absent");
check(tauriConfig.bundle?.windows?.timestampUrl == null, "dev timestamp URL must be absent");
check(tauriConfig.bundle?.windows?.signCommand == null, "dev sign command must be absent");
check(tauriConfig.bundle?.windows?.tsp === false, "dev TSP signing must be false");
check(updateMetadata.enabled === false, "update metadata placeholder must stay disabled");
check(updateMetadata.channel === "dev-local", "update metadata placeholder must use dev-local channel");
check(existsSync(join(desktopRoot, "src-tauri", "src", "release.rs")), "support bundle release module must exist");
check(existsSync(join(desktopRoot, "src-tauri", "icons", "icon.ico")), "Windows icon.ico must exist for bundling");
check(cargoToml.includes("tauri ="), "Cargo.toml must include the Tauri runtime dependency");
check(cargoToml.includes("tauri-build"), "Cargo.toml must include tauri-build for packaging resources");
check(tauriMain.includes("tauri::Builder"), "Rust main must launch the Tauri runtime");

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("Release smoke passed: Windows NSIS dev packaging, unsigned signing checks, and update metadata placeholder are clear.");

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function check(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}

function bundleTargets(config) {
  const targets = config.bundle?.targets ?? [];
  return Array.isArray(targets) ? targets : [targets];
}
