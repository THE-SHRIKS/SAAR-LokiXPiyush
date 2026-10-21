import time
import os
import psutil
from paddleocr import PaddleOCR

def measure_ocr(img_path):
    process = psutil.Process(os.getpid())
    mem_before = process.memory_info().rss / (1024 * 1024)
    
    print(f"Initializing PaddleOCR (lang='en', CPU only)...")
    t0 = time.perf_counter()
    ocr = PaddleOCR(use_textline_orientation=True, lang='en')
    init_time = time.perf_counter() - t0
    print(f"PaddleOCR initialized in {init_time:.2f}s")
    
    t1 = time.perf_counter()
    results = ocr.predict(img_path)
    infer_time = time.perf_counter() - t1
    
    mem_after = process.memory_info().rss / (1024 * 1024)
    print(f"OCR inference completed in {infer_time:.2f}s")
    print(f"Memory RSS before: {mem_before:.1f} MB, after: {mem_after:.1f} MB, diff: {mem_after - mem_before:.1f} MB")
    
    print("\n--- Extracted Text ---")
    line_count = 0
    for res in results:
        # Check standard properties in PaddleX/PaddleOCR result object
        if hasattr(res, 'keys'):
            rec_texts = res.get('rec_texts', [])
            rec_scores = res.get('rec_scores', [])
            dt_polys = res.get('dt_polys', [])
            for text, score, poly in zip(rec_texts, rec_scores, dt_polys):
                line_count += 1
                print(f"[{line_count:02d}] (Conf: {score:.3f}) {text}")
        elif isinstance(res, dict):
            for k, v in res.items():
                print(f"{k}: {v}")
        else:
            print(str(res))

if __name__ == "__main__":
    test_img = "tests/fixtures/receipts/thermal_receipt_scanned.png"
    print(f"Testing OCR on: {test_img}")
    measure_ocr(test_img)
