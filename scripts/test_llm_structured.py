import subprocess
import json
import time
import os
import psutil
import jsonschema

ROOT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LLAMA_CLI_BIN = os.path.join(ROOT_DIR, "bin", "llama", "llama-cli.exe" if os.name == "nt" else "llama-cli")
MODEL_PATH = os.path.join(ROOT_DIR, "models", "weights", "Qwen2.5-3B-Instruct-Q3_K_M.gguf")
SCHEMA_PATH = os.path.join(ROOT_DIR, "scripts", "invoice_schema.json")
GRAMMAR_PATH = os.path.join(ROOT_DIR, "scripts", "invoice_grammar.gbnf")

INPUT_RECEIPT_TEXT = """
OFFICE SUPPLIES MART
Store #402 - Main St Branch
Tel (555) 019-2834
DATE: 2026-09-10 TIME: 14:22:10
RECEIPT ID: REC-99214

1 A4 Printing Paper Ream: $15.00
1 Gel Pens Pack (Black): $10.50
1 USB-C HighSpeed Cable: $20.00

SUBTOTAL: $45.50
TAX (0%): $0.00
TOTAL: $45.50

PAID: VISA ending 4092
AUTH CODE: 092184
THANK YOU FOR YOUR VISIT!
"""

SYSTEM_PROMPT = """You are an accurate financial data extraction assistant.
Extract the transaction details into JSON conforming to the schema.
Amounts MUST be in exact minor units (integer cents). E.g. $15.00 -> 1500, $45.50 -> 4550.
Do NOT use floating point numbers."""

USER_PROMPT = f"""Extract fields from this receipt:
\"\"\"{INPUT_RECEIPT_TEXT.strip()}\"\"\""""

def test_inference():
    assert os.path.exists(LLAMA_CLI_BIN), f"Binary not found: {LLAMA_CLI_BIN}"
    assert os.path.exists(MODEL_PATH), f"Model not found: {MODEL_PATH}"
    assert os.path.exists(SCHEMA_PATH), f"Schema not found: {SCHEMA_PATH}"
    assert os.path.exists(GRAMMAR_PATH), f"Grammar not found: {GRAMMAR_PATH}"

    with open(SCHEMA_PATH, "r") as f:
        schema = json.load(f)

    full_prompt = (
        f"<|im_start|>system\n{SYSTEM_PROMPT}<|im_end|>\n"
        f"<|im_start|>user\n{USER_PROMPT}<|im_end|>\n"
        f"<|im_start|>assistant\n"
    )

    cmd = [
        LLAMA_CLI_BIN,
        "-m", MODEL_PATH,
        "-p", full_prompt,
        "--grammar-file", GRAMMAR_PATH,
        "-c", "2048",
        "-t", "2",           # Restrict to 2 threads for 2-core CPU
        "-n", "300",
        "-st",               # Single turn conversation
        "--temp", "0.0",     # Deterministic greedy sampling
        "--simple-io",
        "-ngl", "0"          # CPU only
    ]

    print("Executing llama-cli CPU inference with GBNF grammar constraint...")
    print(f"Command: {' '.join(cmd[:6])} ...")

    env = os.environ.copy()
    build_bin = os.path.dirname(LLAMA_CLI_BIN)
    ld_paths = [build_bin, "/home/theshriksloki/OpenDeck/.local-sys/usr/lib/x86_64-linux-gnu", "/usr/lib/x86_64-linux-gnu"]
    if "LD_LIBRARY_PATH" in env and env["LD_LIBRARY_PATH"]:
        ld_paths.append(env["LD_LIBRARY_PATH"])
    env["LD_LIBRARY_PATH"] = ":".join(ld_paths)

    t0 = time.perf_counter()
    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env
    )
    
    stdout, stderr = proc.communicate()
    elapsed = time.perf_counter() - t0

    print(f"\n--- llama.cpp Process Exited in {elapsed:.2f}s with code {proc.returncode} ---")
    
    # Parse generation speed and tokens from stderr / stdout
    combined_output = stdout + "\n" + stderr
    for line in combined_output.splitlines():
        if "Prompt:" in line and "Generation:" in line:
            print(f"[Timing Metric] {line.strip()}")
        elif "eval time =" in line or "load time =" in line or "total time =" in line:
            print(f"[stderr timing] {line.strip()}")

    # Find the JSON payload in stdout
    # Extract text between '{' and '}'
    start_idx = stdout.find("{")
    end_idx = stdout.rfind("}")
    if start_idx == -1 or end_idx == -1:
        print(f"Raw stdout:\n{stdout}")
        print(f"Raw stderr:\n{stderr}")
        raise ValueError("Could not find JSON object in stdout")

    json_str = stdout[start_idx:end_idx + 1]
    print("\n--- Extracted JSON Payload ---")
    print(json_str)

    # Validate JSON structure
    try:
        data = json.loads(json_str)
        print("\n--- JSON Schema Validation ---")
        jsonschema.validate(instance=data, schema=schema)
        print("Schema validation: PASSED!")
        print(f"Extracted Vendor: {data.get('vendor')}")
        print(f"Extracted Date: {data.get('date')}")
        print(f"Extracted Receipt ID: {data.get('receipt_id')}")
        print(f"Extracted Total Cents: {data.get('total_cents')} (type: {type(data.get('total_cents')).__name__})")
        print(f"Extracted Tax Cents: {data.get('tax_cents')} (type: {type(data.get('tax_cents')).__name__})")
        print(f"Extracted Items Count: {len(data.get('line_items', []))}")
        for idx, item in enumerate(data.get("line_items", [])):
            print(f"  Item {idx+1}: {item['description']} -> {item['amount_cents']} cents")
        
        # Verify integer arithmetic
        computed_sum = sum(item["amount_cents"] for item in data.get("line_items", []))
        tax = data.get("tax_cents", 0) or 0
        total = data.get("total_cents")
        print(f"\n--- Deterministic Currency Arithmetic Check ---")
        print(f"Items sum: {computed_sum} + tax: {tax} = {computed_sum + tax} vs total: {total}")
        assert computed_sum + tax == total, f"Arithmetic check failed: {computed_sum + tax} != {total}"
        print("Integer arithmetic verification: EXACT MATCH (0 rounding error)!")

    except Exception as e:
        print(f"Failed to validate JSON: {e}")
        raise

if __name__ == "__main__":
    test_inference()
