# Third-Party Model & Runtime Licenses

All local machine learning models and inference runtimes bundled or executed by OpenDeck are distributed under permissive open-source licenses permitting commercial and offline use.

---

### 1. Qwen2.5 Language Models (Qwen2.5-3B-Instruct)
- **Copyright:** Alibaba Cloud / Qwen Team
- **License:** Apache License 2.0
- **License Text Summary:** Subject to the terms and conditions of Apache 2.0, Alibaba grants each user a perpetual, worldwide, non-exclusive, no-charge, royalty-free, irrevocable copyright license to reproduce, prepare derivative works of, publicly display, publicly perform, sublicense, and distribute the work.
- **Quantization:** Quantized by bartowski using llama.cpp standard k-quants.
- **Upstream Origin:** https://huggingface.co/Qwen/Qwen2.5-3B-Instruct

---

### 2. llama.cpp Inference Runtime
- **Copyright:** Georgi Gerganov and llama.cpp contributors
- **License:** MIT License
- **License Text Summary:** Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files, to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies.
- **Upstream Origin:** https://github.com/ggerganov/llama.cpp

---

### 3. RapidOCR & PaddleOCR Models
- **Copyright:** PaddleOCR Authors / RapidAI Contributors
- **License:** Apache License 2.0
- **Models Included:**
  - `ch_PP-OCRv4_det_infer.onnx` (Detection)
  - `ch_ppocr_mobile_v2.0_cls_infer.onnx` (Direction Classification)
  - `en_PP-OCRv4_rec_infer.onnx` (English Text Recognition)
- **Upstream Origin:** https://github.com/RapidAI/RapidOCR and https://github.com/PaddlePaddle/PaddleOCR

---

### 4. ONNX Runtime
- **Copyright:** Microsoft Corporation
- **License:** MIT License
- **Upstream Origin:** https://github.com/microsoft/onnxruntime
