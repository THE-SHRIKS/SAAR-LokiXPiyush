/**
 * Native Edge Headless DevTools Automation to capture real SAAR UI screenshots
 */

import { spawn } from "child_process";
import fs from "fs";
import path from "path";

const EDGE_PATH = "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe";
const OUTPUT_DIR = path.resolve("Docs/screenshots");

if (!fs.existsSync(OUTPUT_DIR)) {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

async function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function run() {
  console.log("Launching Edge in headless remote-debugging mode...");
  const edge = spawn(EDGE_PATH, [
    "--headless=new",
    "--remote-debugging-port=9222",
    "--disable-gpu",
    "--no-first-run",
    "--no-default-browser-check",
    "http://localhost:1420",
  ]);

  await sleep(2500);

  try {
    const listRes = await fetch("http://127.0.0.1:9222/json/list");
    const targets = await listRes.json();
    const target = targets.find((t) => t.type === "page") || targets[0];
    if (!target) throw new Error("No page target found");

    const ws = new WebSocket(target.webSocketDebuggerUrl);

    let messageId = 1;
    const pendingPromises = new Map();

    ws.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      if (msg.id && pendingPromises.has(msg.id)) {
        const { resolve, reject } = pendingPromises.get(msg.id);
        pendingPromises.delete(msg.id);
        if (msg.error) reject(msg.error);
        else resolve(msg.result);
      }
    };

    function send(method, params = {}) {
      return new Promise((resolve, reject) => {
        const id = messageId++;
        pendingPromises.set(id, { resolve, reject });
        ws.send(JSON.stringify({ id, method, params }));
      });
    }

    await new Promise((resolve) => {
      ws.onopen = resolve;
    });

    console.log("Connected to DevTools Protocol.");

    await send("Page.enable");
    await send("Runtime.enable");

    async function setViewport(width, height) {
      await send("Emulation.setDeviceMetricsOverride", {
        width,
        height,
        deviceScaleFactor: 1,
        mobile: false,
      });
      await send("Emulation.setVisibleSize", { width, height });
    }

    async function takeScreenshot(fileName) {
      await sleep(700);
      const res = await send("Page.captureScreenshot", { format: "png" });
      const buffer = Buffer.from(res.data, "base64");
      const filePath = path.join(OUTPUT_DIR, fileName);
      fs.writeFileSync(filePath, buffer);
      console.log(`Saved screenshot: ${fileName} (${buffer.length} bytes)`);
    }

    async function evaluate(expression) {
      const res = await send("Runtime.evaluate", {
        expression,
        returnByValue: true,
        awaitPromise: true,
      });
      return res.result?.value;
    }

    // 1. Locked Screen at 1280x800
    await setViewport(1280, 800);
    await sleep(800);
    await takeScreenshot("01_locked_screen_1280.png");

    // Locked Screen at 960x700
    await setViewport(960, 700);
    await sleep(500);
    await takeScreenshot("11_locked_screen_960x700.png");

    // Reset to 1280x800 and Unlock
    await setViewport(1280, 800);
    await sleep(500);
    console.log("Unlocking workspace...");
    await evaluate(`document.querySelector(".btn-demo-quick")?.click();`);
    await sleep(1000);

    // 2. Chat Workspace (Primary Default)
    console.log("Capturing 02_chat_workspace_1280.png...");
    await takeScreenshot("02_chat_workspace_1280.png");

    // 3. Contextual Evidence Inspector Open
    console.log("Opening Evidence Inspector via citation click...");
    await evaluate(`
      const cite = document.querySelector(".citation-pill-btn");
      if (cite) cite.click();
    `);
    await sleep(800);
    await takeScreenshot("03_evidence_inspector_open_1280.png");

    // Close Inspector for full view of next tabs
    await evaluate(`
      const closeBtn = document.querySelector(".inspector-close-btn");
      if (closeBtn) closeBtn.click();
    `);
    await sleep(400);

    // 4. Documents View
    console.log("Navigating to Documents View...");
    await evaluate(`document.querySelector('button[aria-label="Documents"]')?.click();`);
    await sleep(800);
    await takeScreenshot("04_documents_view_1280.png");

    // 5. Reconciliation & Review View
    console.log("Navigating to Review View...");
    await evaluate(`document.querySelector('button[aria-label="Review"]')?.click();`);
    await sleep(800);
    await takeScreenshot("05_reconciliation_review_1280.png");

    // 5b. Ledger Postings subtab
    console.log("Switching to Ledger Postings subtab...");
    await evaluate(`
      const tabs = Array.from(document.querySelectorAll(".subnav-tab"));
      const ledgerTab = tabs.find(t => t.textContent.includes("Ledger"));
      if (ledgerTab) ledgerTab.click();
    `);
    await sleep(800);
    await takeScreenshot("05b_ledger_postings_1280.png");

    // 5c. Audit Trail subtab
    console.log("Switching to Audit Trail subtab...");
    await evaluate(`
      const tabs = Array.from(document.querySelectorAll(".subnav-tab"));
      const auditTab = tabs.find(t => t.textContent.includes("Audit"));
      if (auditTab) auditTab.click();
    `);
    await sleep(800);
    await takeScreenshot("05c_audit_trail_1280.png");

    // 6. Financial Reports View
    console.log("Navigating to Reports View...");
    await evaluate(`document.querySelector('button[aria-label="Reports"]')?.click();`);
    await sleep(800);
    await takeScreenshot("06_financial_reports_1280.png");

    // 7. Security & Offline Runtime View
    console.log("Navigating to Security View...");
    await evaluate(`document.querySelector('button[aria-label="Security"]')?.click();`);
    await sleep(800);
    await takeScreenshot("07_security_runtime_1280.png");

    // 8. Viewport Variations on Chat Workspace
    console.log("Capturing multi-viewport variations...");
    await evaluate(`document.querySelector('button[aria-label="Conversation Workspace"]')?.click();`);
    await sleep(600);

    // 1920x1080
    await setViewport(1920, 1080);
    await sleep(600);
    await takeScreenshot("08_chat_workspace_1920x1080.png");

    // 1366x768
    await setViewport(1366, 768);
    await sleep(600);
    await takeScreenshot("09_chat_workspace_1366x768.png");

    // 960x700
    await setViewport(960, 700);
    await sleep(600);
    await takeScreenshot("10_chat_workspace_960x700.png");

    ws.close();
    console.log("All screenshots captured successfully.");
  } finally {
    edge.kill();
  }
}

run().catch((err) => {
  console.error("Screenshot capture failed:", err);
  process.exit(1);
});
