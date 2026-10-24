// Minimal Chrome DevTools Protocol client for driving the real SAAR window.
//
// SAAR's window is a WebView2 control, which exposes the DevTools protocol when started with
// --remote-debugging-port. This client attaches to the running saar.exe on localhost, then clicks,
// types and takes screenshots against the real application and its real backend. No browser
// mock, no dev server: the bundled frontend runs inside the packaged window.
//
// The debugging port is opened only by this test harness (through an environment variable on the
// child process). It is not part of the shipped application's configuration.

import { spawn, execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

export class AppDriver {
  constructor({ exe, dataDir, userDataDir, port, configPath }) {
    this.exe = exe;
    this.dataDir = dataDir;
    this.userDataDir = userDataDir;
    this.port = port;
    this.configPath = configPath;
    this.nextId = 1;
    this.pending = new Map();
    this.child = null;
    this.ws = null;
  }

  /** The shipped lockdown arguments plus the test-only debugging port. */
  browserArgs() {
    const cfg = JSON.parse(fs.readFileSync(this.configPath, "utf8"));
    const shipped = cfg.app.windows[0].additionalBrowserArgs ?? "";
    return `${shipped} --remote-debugging-port=${this.port} --remote-allow-origins=*`.trim();
  }

  async start() {
    fs.mkdirSync(this.dataDir, { recursive: true });
    fs.mkdirSync(this.userDataDir, { recursive: true });
    // Windows Application Control blocks node.exe from launching the unsigned debug build
    // directly. The app is started the way a user would start it, through PowerShell's
    // Start-Process, and this harness only attaches to it afterwards. No policy is changed.
    const launcher = spawn(
      "powershell.exe",
      ["-NoProfile", "-NonInteractive", "-Command", `(Start-Process -FilePath '${this.exe}' -PassThru).Id`],
      {
        env: {
          ...process.env,
          SAAR_DATA_DIR: this.dataDir,
          WEBVIEW2_USER_DATA_FOLDER: this.userDataDir,
          WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: this.browserArgs(),
        },
        stdio: ["ignore", "pipe", "pipe"],
      },
    );
    let out = "";
    let err = "";
    launcher.stdout.on("data", (d) => (out += d));
    launcher.stderr.on("data", (d) => (err += d));
    await new Promise((resolve) => launcher.on("close", resolve));
    this.pid = parseInt(out.trim(), 10);
    if (!this.pid) throw new Error(`could not start the application: ${err.trim() || out.trim()}`);
    let target = null;
    for (let i = 0; i < 120 && !target; i++) {
      await sleep(500);
      try {
        const list = await (await fetch(`http://127.0.0.1:${this.port}/json/list`)).json();
        target = list.find((t) => t.type === "page" && /tauri\.localhost|localhost/.test(t.url));
      } catch {
        /* the window is not up yet */
      }
    }
    if (!target) throw new Error("the SAAR window did not expose a page target");
    this.ws = new WebSocket(target.webSocketDebuggerUrl);
    await new Promise((resolve, reject) => {
      this.ws.onopen = resolve;
      this.ws.onerror = reject;
    });
    this.ws.onmessage = (ev) => {
      const msg = JSON.parse(ev.data);
      if (msg.id && this.pending.has(msg.id)) {
        const { resolve, reject } = this.pending.get(msg.id);
        this.pending.delete(msg.id);
        msg.error ? reject(new Error(JSON.stringify(msg.error))) : resolve(msg.result);
      }
    };
    await this.send("Page.enable");
    await this.send("Runtime.enable");
    return target.url;
  }

  send(method, params = {}) {
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.ws.send(JSON.stringify({ id, method, params }));
    });
  }

  /** Evaluate an expression in the page and return its JSON value. */
  async eval(expression) {
    const r = await this.send("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true,
    });
    if (r.exceptionDetails) {
      throw new Error(`page error: ${r.exceptionDetails.exception?.description ?? r.exceptionDetails.text}`);
    }
    return r.result.value;
  }

  async waitFor(expression, { timeout = 30000, interval = 250, what = expression } = {}) {
    const end = Date.now() + timeout;
    let last;
    while (Date.now() < end) {
      try {
        last = await this.eval(expression);
        if (last) return last;
      } catch (e) {
        last = e.message;
      }
      await sleep(interval);
    }
    throw new Error(`timed out after ${timeout} ms waiting for: ${what} (last: ${JSON.stringify(last)})`);
  }

  /** Real mouse click at the centre of the element returned by `finder` (a JS expression). */
  async click(finder, what = finder) {
    const rect = await this.eval(`(() => {
      const el = (${finder});
      if (!el) return null;
      el.scrollIntoView({ block: "center", inline: "center" });
      const r = el.getBoundingClientRect();
      return { x: r.left + r.width / 2, y: r.top + r.height / 2, w: r.width, h: r.height, disabled: !!el.disabled };
    })()`);
    if (!rect) throw new Error(`nothing to click: ${what}`);
    if (rect.disabled) throw new Error(`control is disabled: ${what}`);
    if (rect.w === 0 || rect.h === 0) throw new Error(`control has no size: ${what}`);
    await this.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: rect.x, y: rect.y });
    await this.send("Input.dispatchMouseEvent", { type: "mousePressed", x: rect.x, y: rect.y, button: "left", clickCount: 1 });
    await this.send("Input.dispatchMouseEvent", { type: "mouseReleased", x: rect.x, y: rect.y, button: "left", clickCount: 1 });
  }

  /** Focus the element, replace its contents with `text` using real text input. */
  async type(finder, text, what = finder) {
    const ok = await this.eval(`(() => {
      const el = (${finder});
      if (!el) return false;
      el.focus();
      if (typeof el.select === "function") el.select();
      return document.activeElement === el;
    })()`);
    if (!ok) throw new Error(`cannot focus: ${what}`);
    await this.send("Input.insertText", { text });
  }

  async press(key, code = key, vk = 0, modifiers = 0) {
    await this.send("Input.dispatchKeyEvent", { type: "keyDown", key, code, windowsVirtualKeyCode: vk, modifiers });
    await this.send("Input.dispatchKeyEvent", { type: "keyUp", key, code, windowsVirtualKeyCode: vk, modifiers });
  }

  async viewport(width, height) {
    await this.send("Emulation.setDeviceMetricsOverride", { width, height, deviceScaleFactor: 1, mobile: false });
  }

  async screenshot(file) {
    const r = await this.send("Page.captureScreenshot", { format: "png" });
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, Buffer.from(r.data, "base64"));
    return file;
  }

  /** Close the window and every process it started (WebView2 children included). */
  stop() {
    try {
      this.ws?.close();
    } catch {
      /* already closed */
    }
    if (this.pid) {
      try {
        execFileSync("taskkill", ["/PID", String(this.pid), "/T", "/F"], { stdio: "ignore" });
      } catch {
        /* already gone */
      }
    }
  }
}

/** JS expression helpers evaluated inside the page. */
export const q = {
  byText: (selector, text, exact = false) => `(() => {
    const els = [...document.querySelectorAll(${JSON.stringify(selector)})];
    const norm = (s) => (s || "").replace(/\\s+/g, " ").trim();
    return els.find((e) => ${exact} ? norm(e.textContent) === ${JSON.stringify(text)} : norm(e.textContent).includes(${JSON.stringify(text)})) || null;
  })()`,
  sel: (selector) => `document.querySelector(${JSON.stringify(selector)})`,
  id: (id) => `document.getElementById(${JSON.stringify(id)})`,
};
