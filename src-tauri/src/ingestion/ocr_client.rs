use crate::supervisor::{SupervisorError, WorkerSupervisor};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrLine {
    pub text: String,
    pub confidence: f64,
    pub bbox: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrPage {
    pub page_num: i32,
    pub width: f64,
    pub height: f64,
    pub lines: Vec<OcrLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResponse {
    pub status: String,
    #[serde(default)]
    pub tier: String,
    #[serde(default)]
    pub page_count: i32,
    #[serde(default)]
    pub pages: Vec<OcrPage>,
    pub error: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum OcrError {
    #[error("Supervisor error: {0}")]
    Supervisor(#[from] SupervisorError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON deserialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("OCR worker reported error: {0}")]
    WorkerReported(String),
    #[error("Python OCR runtime executable not found")]
    RuntimeNotFound,
}

pub struct OcrClient {
    python_bin: Option<PathBuf>,
    standalone_bin: Option<PathBuf>,
    worker_script: PathBuf,
    timeout: Duration,
}

impl OcrClient {
    pub fn new() -> Result<Self, OcrError> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();

        let standalone_candidates = [
            root.join("workers/ocr/dist/ocr_worker.exe"),
            root.join("workers/ocr/dist/ocr_worker"),
            root.join("bin/ocr/ocr_worker.exe"),
            root.join("bin/ocr/ocr_worker"),
        ];

        let standalone_bin = standalone_candidates.into_iter().find(|p| p.exists());

        let candidate_bins = vec![
            root.join(".bin/python3"),
            root.join(".bin/python.exe"),
            PathBuf::from("/home/theshriksloki/OpenDeck/.bin/python3"),
            PathBuf::from("/usr/bin/python3"),
            PathBuf::from("python3"),
            PathBuf::from("python"),
            PathBuf::from("python.exe"),
        ];

        let python_bin = candidate_bins.into_iter().find(|p| p.exists());

        if standalone_bin.is_none() && python_bin.is_none() {
            return Err(OcrError::RuntimeNotFound);
        }

        let worker_script = root.join("workers/ocr/ocr_worker.py");

        Ok(Self {
            python_bin,
            standalone_bin,
            worker_script,
            timeout: Duration::from_secs(30),
        })
    }

    /// Executes OCR worker against a file path under supervision
    pub fn extract_document(&self, file_path: &Path) -> Result<OcrResponse, OcrError> {
        let file_path_str = file_path
            .to_str()
            .ok_or_else(|| OcrError::WorkerReported("Invalid file path encoding".to_string()))?;

        let (exec_cmd, args_vec) = if let Some(ref standalone) = self.standalone_bin {
            (
                standalone.to_string_lossy().to_string(),
                vec!["--input-file".to_string(), file_path_str.to_string()],
            )
        } else if let Some(ref py) = self.python_bin {
            (
                py.to_string_lossy().to_string(),
                vec![
                    self.worker_script.to_string_lossy().to_string(),
                    "--input-file".to_string(),
                    file_path_str.to_string(),
                ],
            )
        } else {
            return Err(OcrError::RuntimeNotFound);
        };

        let args_refs: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();

        let mut child_guard = WorkerSupervisor::spawn_worker(&exec_cmd, &args_refs)?;

        // Wait with bounded timeout
        let status = child_guard.wait_with_timeout(self.timeout)?;

        // Read output from the worker process (re-executing standard output collection)
        let output = std::process::Command::new(&exec_cmd)
            .args(&args_refs)
            .output()?;

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let resp: OcrResponse = serde_json::from_str(stdout_str.trim())?;

        if !status.success() || resp.status != "success" {
            let err_msg = resp
                .error
                .unwrap_or_else(|| "Worker process failed".to_string());
            return Err(OcrError::WorkerReported(err_msg));
        }

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_digital_pdf_fixture() {
        let client = OcrClient::new().expect("Failed to initialize OcrClient");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("tests/fixtures/invoices/gst_b2b_valid.pdf");

        let resp = client.extract_document(&path).expect("Extraction failed");
        assert_eq!(resp.status, "success");
        assert_eq!(resp.tier, "digital");
        assert_eq!(resp.page_count, 1);
        assert!(!resp.pages.is_empty());

        let lines = &resp.pages[0].lines;
        let has_supplier = lines.iter().any(|l| l.text.contains("ACME Tech Services"));
        assert!(
            has_supplier,
            "Digital text extraction must extract ACME Tech Services"
        );
    }

    #[test]
    fn test_corrupted_pdf_returns_recoverable_error() {
        let client = OcrClient::new().expect("Failed to initialize OcrClient");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("tests/fixtures/adversarial/corrupted_truncated.pdf");

        let res = client.extract_document(&path);
        assert!(res.is_err(), "Corrupted PDF must return error");
        match res.unwrap_err() {
            OcrError::WorkerReported(msg) => {
                assert!(
                    msg.contains("Failed to load")
                        || msg.contains("PDFium")
                        || msg.contains("format error")
                );
            }
            other => panic!("Expected WorkerReported error, got {:?}", other),
        }
    }
}
