const puppeteer = require("puppeteer-core");
const path = require("path");
const fs = require("fs");

const EDGE_PATH = "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe";
const OUT_DIR = path.resolve(__dirname, "../Docs/screenshots");

async function main() {
  if (!fs.existsSync(OUT_DIR)) {
    fs.mkdirSync(OUT_DIR, { recursive: true });
  }

  console.log("Launching Edge from:", EDGE_PATH);
  const browser = await puppeteer.launch({
    executablePath: EDGE_PATH,
    headless: "new",
    args: ["--no-sandbox", "--disable-setuid-sandbox", "--disable-gpu", "--window-size=1366,840"],
    defaultViewport: { width: 1366, height: 840 },
  });

  const page = await browser.newPage();

  console.log("Navigating to http://localhost:1420...");
  await page.goto("http://localhost:1420", { waitUntil: "networkidle0" });

  // 1. Dark Unlock Screen
  console.log("1. Capturing dark_01_unlock.png...");
  await page.screenshot({ path: path.join(OUT_DIR, "dark_01_unlock.png") });

  // 2. Toggle to Linen light theme on Unlock Screen
  const unlockToggle = await page.$(".unlock-theme-toggle");
  if (unlockToggle) {
    console.log("2. Toggling to Linen light theme on unlock screen...");
    await unlockToggle.click();
    await new Promise((r) => setTimeout(r, 600));
    await page.screenshot({ path: path.join(OUT_DIR, "light_01_unlock.png") });
    await page.screenshot({ path: path.join(OUT_DIR, "01_unlock_screen.png") });
  }

  // 3. Click Quick Unlock (Demo Vault)
  console.log("3. Clicking Quick Unlock...");
  const quickUnlockBtn = await page.$(".quick-demo-btn");
  if (quickUnlockBtn) {
    await quickUnlockBtn.click();
    await page.waitForSelector(".claude-desktop-shell", { timeout: 10000 });
    await new Promise((r) => setTimeout(r, 1000));
  }

  // 4. Capture Clean Initial Hero State (No fake preloaded conversations)
  console.log("4. Capturing 03_chat_empty_hero.png (Clean Hero State)...");
  await page.screenshot({ path: path.join(OUT_DIR, "03_chat_empty_hero.png") });
  await page.screenshot({ path: path.join(OUT_DIR, "light_03_chat_empty_hero.png") });

  // 5. Send Real Query to Local AI Model
  console.log("5. Sending prompt to local Qwen model...");
  const textarea = await page.$(".composer-textarea");
  if (textarea) {
    await textarea.type("What is our net operating surplus and total expense?");
    await new Promise((r) => setTimeout(r, 300));
    const sendBtn = await page.$(".composer-send-btn.ready");
    if (sendBtn) {
      await sendBtn.click();
      console.log("Waiting for real local model inference...");
      // Wait for assistant response to appear
      await page.waitForSelector(".assistant-message-entry:not(.loading)", { timeout: 45000 });
      await new Promise((r) => setTimeout(r, 1200));
      await page.screenshot({ path: path.join(OUT_DIR, "02_chat_seed_report.png") });
      await page.screenshot({ path: path.join(OUT_DIR, "light_02_chat_seed_report.png") });
      console.log("Captured real model response in 02_chat_seed_report.png");
    }
  }

  // 6. Open Evidence Inspector
  console.log("6. Toggling Evidence Inspector...");
  const inspectorBtn = await page.$(".inspector-toggle-btn");
  if (inspectorBtn) {
    await inspectorBtn.click();
    await new Promise((r) => setTimeout(r, 600));
    await page.screenshot({ path: path.join(OUT_DIR, "04_evidence_inspector.png") });
    await page.screenshot({ path: path.join(OUT_DIR, "light_04_evidence_inspector.png") });
    await inspectorBtn.click();
    await new Promise((r) => setTimeout(r, 300));
  }

  // 7. Navigate to Documents & Ingestion
  console.log("7. Navigating to Documents & Ingestion...");
  let toolBtns = await page.$$(".tool-nav-btn");
  if (toolBtns.length >= 1) {
    await toolBtns[0].click();
    await new Promise((r) => setTimeout(r, 600));
    await page.screenshot({ path: path.join(OUT_DIR, "04_documents_archive.png") });
    await page.screenshot({ path: path.join(OUT_DIR, "light_04_documents_archive.png") });
  }

  // 8. Navigate to Reconciliation & Ledger Review
  console.log("8. Navigating to Reconciliation & Ledger Review...");
  if (toolBtns.length >= 2) {
    await toolBtns[1].click();
    await new Promise((r) => setTimeout(r, 600));
    await page.screenshot({ path: path.join(OUT_DIR, "05_reconciliation_review.png") });
    await page.screenshot({ path: path.join(OUT_DIR, "light_05_reconciliation_review.png") });
  }

  // 9. Navigate to Financial Reports
  console.log("9. Navigating to Financial Reports...");
  if (toolBtns.length >= 3) {
    await toolBtns[2].click();
    await new Promise((r) => setTimeout(r, 600));
    await page.screenshot({ path: path.join(OUT_DIR, "06_financial_reports.png") });
    await page.screenshot({ path: path.join(OUT_DIR, "light_06_financial_reports.png") });
  }

  // 10. Switch to Dark Mode
  console.log("10. Switching to Midnight Navy Dark Mode...");
  const themeToggle = await page.$(".theme-toggle-btn");
  if (themeToggle) {
    await themeToggle.click();
    await new Promise((r) => setTimeout(r, 600));
    await page.screenshot({ path: path.join(OUT_DIR, "dark_06_financial_reports.png") });
  }

  // 11. Navigate back to Chat in Dark Mode
  console.log("11. Navigating to Chat in Dark Mode...");
  const threadItems = await page.$$(".thread-item");
  if (threadItems.length >= 1) {
    await threadItems[0].click();
    await new Promise((r) => setTimeout(r, 600));
    await page.screenshot({ path: path.join(OUT_DIR, "dark_02_chat_seed_report.png") });
  }

  await browser.close();
  console.log("All screenshots captured successfully in:", OUT_DIR);
}

main().catch((err) => {
  console.error("Screenshot error:", err);
  process.exit(1);
});
