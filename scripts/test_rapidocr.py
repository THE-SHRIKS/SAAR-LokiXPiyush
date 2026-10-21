import time
import os
import psutil
from rapidocr_onnxruntime import RapidOCR

def measure_ocr(img_path):
    process = psutil.Process(os.getpid())
    mem_before = process.memory_info().rss / (1024 * 1024)
    
    print(f"Initializing RapidOCR (ONNXRuntime CPU, AVX2)...")
    t0 = time.perf_counter()
    engine = RapidOCR()
    init_time = time.perf_counter() - t0
    print(f"RapidOCR initialized in {init_time:.3f}s")
    
    t1 = time.perf_counter()
    result, elapse_list = engine(img_path)
    infer_time = time.perf_counter() - t1
    
    mem_after = process.memory_info().rss / (1024 * 1024)
    print(f"OCR inference completed in {infer_time:.3f}s (Engine elapsed: {elapse_list})")
    print(f"Memory RSS before: {mem_before:.1f} MB, after: {mem_after:.1f} MB, diff: {mem_after - mem_before:.1f} MB")
    
    print("\n--- Extracted Text ---")
    if result:
        for idx, line in enumerate(result, 1):
            bbox, text, score = line
            print(f"[{idx:02d}] (Conf: {score:.3f}) {text} | BBox: {bbox}")
    else:
        print("No text detected.")

if __name__ == "__main__":
    test_img = "tests/fixtures/receipts/thermal_receipt_scanned.png"
    print(f"Testing RapidOCR on: {test_img}")
    measure_ocr(test_img)
