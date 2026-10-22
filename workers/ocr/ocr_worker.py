#!/usr/bin/env python3
"""
SAAR Local Document Extraction Worker
Runs offline on CPU with AVX2.
Extracts digital text layers from born-digital PDFs, or runs RapidOCR on scanned documents/images.
Outputs structured JSON to stdout.
"""

import argparse
import json
import os
import sys
import tempfile

def process_digital_pdf(file_path):
    """Tier A: Fast digital text layer extraction with bounding boxes using pdfplumber."""
    import pdfplumber

    pages_data = []
    with pdfplumber.open(file_path) as pdf:
        total_pages = len(pdf.pages)
        if total_pages == 0:
            return {"status": "error", "error": "PDF contains 0 pages.", "pages": []}

        # Guard against decompression bombs / giant files
        if total_pages > 50:
            return {
                "status": "error",
                "error": f"PDF exceeds maximum page limit (50 pages max, found {total_pages}).",
                "pages": [],
            }

        total_words_extracted = 0
        for page_idx, page in enumerate(pdf.pages, start=1):
            width = float(page.width)
            height = float(page.height)
            words = page.extract_words(keep_blank_chars=False)
            total_words_extracted += len(words)

            # Group words by vertical line position (within 3pt tolerance)
            lines = []
            if words:
                sorted_words = sorted(words, key=lambda w: (round(w["top"] / 3) * 3, w["x0"]))
                current_line_words = [sorted_words[0]]
                current_top = sorted_words[0]["top"]

                for w in sorted_words[1:]:
                    if abs(w["top"] - current_top) <= 4.0:
                        current_line_words.append(w)
                    else:
                        line_text = " ".join(item["text"] for item in current_line_words)
                        min_x = min(item["x0"] for item in current_line_words)
                        min_y = min(item["top"] for item in current_line_words)
                        max_x = max(item["x1"] for item in current_line_words)
                        max_y = max(item["bottom"] for item in current_line_words)
                        lines.append({
                            "text": line_text,
                            "confidence": 1.0,
                            "bbox": [float(min_x), float(min_y), float(max_x), float(max_y)],
                        })
                        current_line_words = [w]
                        current_top = w["top"]

                if current_line_words:
                    line_text = " ".join(item["text"] for item in current_line_words)
                    min_x = min(item["x0"] for item in current_line_words)
                    min_y = min(item["top"] for item in current_line_words)
                    max_x = max(item["x1"] for item in current_line_words)
                    max_y = max(item["bottom"] for item in current_line_words)
                    lines.append({
                        "text": line_text,
                        "confidence": 1.0,
                        "bbox": [float(min_x), float(min_y), float(max_x), float(max_y)],
                    })

            pages_data.append({
                "page_num": page_idx,
                "width": width,
                "height": height,
                "lines": lines,
            })

    # If the PDF has virtually no text layer (< 5 words across document), fall back to OCR
    if total_words_extracted < 5:
        return None  # Signal to fallback to OCR

    return {
        "status": "success",
        "tier": "digital",
        "page_count": len(pages_data),
        "pages": pages_data,
        "error": None,
    }

def process_ocr_image_or_scanned_pdf(file_path):
    """Tier B: Local OCR inference using RapidOCR on CPU."""
    from rapidocr_onnxruntime import RapidOCR
    from PIL import Image

    engine = RapidOCR()
    pages_data = []

    ext = os.path.splitext(file_path)[1].lower()
    if ext == ".pdf":
        import pypdfium2 as pdfium
        pdf = pdfium.PdfDocument(file_path)
        page_count = len(pdf)
        if page_count > 20:
            return {
                "status": "error",
                "error": f"Scanned PDF exceeds maximum OCR page limit (20 pages max, found {page_count}).",
                "pages": [],
            }

        with tempfile.TemporaryDirectory() as tmp_dir:
            for page_idx, page in enumerate(pdf, start=1):
                image = page.render(scale=2.0).to_pil()
                width, height = image.size
                tmp_img_path = os.path.join(tmp_dir, f"page_{page_idx}.png")
                image.save(tmp_img_path)

                result, _ = engine(tmp_img_path)
                lines = []
                if result:
                    for item in result:
                        bbox, text, score = item
                        lines.append({
                            "text": str(text),
                            "confidence": round(float(score), 4),
                            "bbox": bbox,
                        })

                pages_data.append({
                    "page_num": page_idx,
                    "width": float(width),
                    "height": float(height),
                    "lines": lines,
                })
    else:
        # Standard image (PNG, JPG, TIFF, etc.)
        with Image.open(file_path) as img:
            width, height = img.size

        result, _ = engine(file_path)
        lines = []
        if result:
            for item in result:
                bbox, text, score = item
                lines.append({
                    "text": str(text),
                    "confidence": round(float(score), 4),
                    "bbox": bbox,
                })

        pages_data.append({
            "page_num": 1,
            "width": float(width),
            "height": float(height),
            "lines": lines,
        })

    return {
        "status": "success",
        "tier": "ocr",
        "page_count": len(pages_data),
        "pages": pages_data,
        "error": None,
    }

def main():
    parser = argparse.ArgumentParser(description="SAAR Local Document Extractor")
    parser.add_argument("--input-file", required=True, help="Path to input document file")
    parser.add_argument(
        "--mode",
        choices=["auto", "digital", "ocr"],
        default="auto",
        help="Extraction strategy mode",
    )
    args = parser.parse_args()

    input_path = args.input_file
    if not os.path.isfile(input_path):
        out = {
            "status": "error",
            "tier": "unknown",
            "page_count": 0,
            "pages": [],
            "error": f"File not found: {input_path}",
        }
        print(json.dumps(out))
        sys.exit(1)

    try:
        ext = os.path.splitext(input_path)[1].lower()

        if ext == ".pdf" and args.mode in ("auto", "digital"):
            try:
                res = process_digital_pdf(input_path)
                if res is not None:
                    print(json.dumps(res))
                    sys.exit(0)
            except Exception as e:
                if args.mode == "digital":
                    raise e
                # In auto mode, fallback to OCR if digital parser fails

        # Fallback or explicit OCR
        res = process_ocr_image_or_scanned_pdf(input_path)
        print(json.dumps(res))
        sys.exit(0)

    except Exception as e:
        out = {
            "status": "error",
            "tier": "error",
            "page_count": 0,
            "pages": [],
            "error": str(e),
        }
        print(json.dumps(out))
        sys.exit(1)

if __name__ == "__main__":
    main()
