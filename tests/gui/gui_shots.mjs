// Visual review helper: opens the real window on a populated synthetic workspace and saves a
// screenshot of every screen in both themes. Not a pass/fail test.
//
//   node tests/gui/gui_shots.mjs [out-dir]

import { AppDriver, q } from "./cdp.mjs";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "..", "..");
const exe = path.join(repo, "src-tauri", "target", "debug", "saar.exe");
const corpus = path.join(repo, "tests", "fixtures", "corpus", "workspace_beta");
const out = path.resolve(process.argv[2] ?? path.join(os.tmpdir(), "saar_shots"));
fs.mkdirSync(out, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const root = fs.mkdtempSync(path.join(os.tmpdir(), "saar_shots_"));
const app = new AppDriver({
  exe,
  dataDir: path.join(root, "data"),
  userDataDir: path.join(root, "webview"),
  port: 9361,
  configPath: path.join(repo, "src-tauri", "tauri.conf.json"),
});

async function ingest(file) {
  await app.waitFor(`(() => { const i = document.querySelector("input.input-text.flex-grow"); return !!i && !i.disabled; })()`, { timeout: 120000 });
  await app.type(q.sel("input.input-text.flex-grow"), file);
  await app.click(q.byText("button.btn-primary-medium", "Ingest Document"), "ingest");
  const name = path.basename(file);
  await app.waitFor(`(() => { const b = document.querySelector(".ingest-status-banner"); const t = b ? b.innerText : ""; return (t.includes(${JSON.stringify(name)}) && !t.startsWith("Ingesting")) || t.startsWith("Ingestion failed"); })()`, { timeout: 120000 });
}

const tabs = [
  ["chat", null],
  ["documents", "Documents"],
  ["review", "Reconciliation"],
  ["reports", "Reports"],
  ["vault", "Security"],
];

try {
  await app.start();
  await app.waitFor(`document.body.innerText.includes("Create encrypted workspace")`, { timeout: 30000 });
  await app.screenshot(path.join(out, "00_unlock_dark.png"));
  await app.type(q.id("workspace-name-input"), "Northwind Operations");
  await app.type(q.id("passphrase-input"), "Shots-Workspace-Passphrase-1");
  await app.type(q.id("passphrase-confirm-input"), "Shots-Workspace-Passphrase-1");
  await app.click(q.sel(".unlock-submit-btn"), "create");
  await app.waitFor(`document.body.innerText.includes("Start new chat")`, { timeout: 60000 });

  await app.click(q.byText("button.tool-nav-btn", "Documents"), "documents");
  const files = [];
  for (const sub of ["invoices", "receipts"]) {
    for (const f of fs.readdirSync(path.join(corpus, sub)).sort().slice(0, 6)) files.push(path.join(corpus, sub, f));
  }
  files.push(path.join(corpus, "bank_statement.csv"));
  for (const f of files) await ingest(f);

  await app.click(q.byText("button.tool-nav-btn", "Reconciliation"), "review");
  await sleep(400);
  await app.click(q.byText("button.btn-primary-medium", "Run Reconciliation"), "reconcile");
  await app.waitFor(`document.body.innerText.includes("Reconciliation complete")`, { timeout: 60000 });

  await app.click(q.sel(".new-chat-btn"), "new chat");
  await app.type(q.sel(".composer-textarea"), "What are my total expenses?");
  await app.click(q.sel(".composer-send-btn"), "send");
  await app.waitFor(`document.querySelectorAll(".chat-message-row.assistant .assistant-message-entry:not(.loading)").length > 0`, { timeout: 120000 });

  await sleep(500);
  await app.screenshot(path.join(out, "conversation.png"));

  for (const theme of ["dark", "light"]) {
    const toggle = await app.eval(`document.querySelector(".saar-app-root")?.getAttribute("data-theme")`);
    if (toggle !== theme) {
      await app.click(q.sel(".theme-toggle-btn, button[title^='Switch to']"), "theme");
      await sleep(300);
    }
    for (const [name, label] of tabs) {
      if (label) await app.click(q.byText("button.tool-nav-btn", label), name);
      else await app.click(q.sel(".new-chat-btn"), name);
      await sleep(600);
      await app.screenshot(path.join(out, `${name}_${theme}.png`));
      if (process.env.SHOTS_PROBE) {
        const probe = await app.eval(`(() => {
          const sels = JSON.parse(${JSON.stringify(process.env.SHOTS_PROBE)});
          return sels.map((s) => { const e = document.querySelector(s); if (!e) return null; const c = getComputedStyle(e); return s + ": color=" + c.color + " bg=" + c.backgroundColor + " op=" + c.opacity; }).filter(Boolean);
        })()`);
        if (probe.length) console.log(theme, name, probe.join(" | "));
      }
    }
  }
  console.log("shots saved to", out);
} finally {
  await app.stop?.();
}
