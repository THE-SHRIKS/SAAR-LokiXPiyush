use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

/// Weights shipped with SAAR: Qwen3-4B, Q4_K_M. The hash is of the file downloaded from the
/// official Qwen/Qwen3-4B-GGUF repository (2,497,280,256 bytes).
pub const MODEL_FILE_NAME: &str = "Qwen3-4B-Q4_K_M.gguf";
pub const MODEL_ID: &str = "qwen3-4b-q4_k_m";
pub const MODEL_LABEL: &str = "Qwen3 4B";
pub const EXPECTED_MODEL_SHA256: &str =
    "7485fe6f11af29433bc51cab58009521f205840f5b4ae3a32fa7f92e8534fdf5";

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("Llama CLI binary not found at path: {0}")]
    BinaryNotFound(PathBuf),
    #[error("Model file not found at path: {0}")]
    ModelNotFound(PathBuf),
    #[error("Model checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("Process execution error: {0}")]
    Process(#[from] std::io::Error),
    #[error("Inference timed out after {0:?}")]
    Timeout(Duration),
    #[error("Inference process failed with code {code:?}: {stderr}")]
    ExecutionFailed { code: Option<i32>, stderr: String },
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub binary_path: PathBuf,
    pub model_path: PathBuf,
    pub thread_count: usize,
    /// Layers offloaded to the GPU. 0 forces CPU. If the GPU run fails, generation retries on CPU.
    pub gpu_layers: usize,
    pub context_size: usize,
    pub max_tokens: usize,
    pub timeout: Duration,
}

impl Default for LlmConfig {
    fn default() -> Self {
        // Resolve paths relative to SAAR workspace root
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();

        let candidate_bins = [
            root.join("bin/llama/llama-cli"),
            root.join("bin/llama/llama-cli.exe"),
            root.join("bin/llama-cli.exe"),
        ];

        let binary_path = candidate_bins
            .into_iter()
            .find(|p| p.exists())
            .unwrap_or_else(|| {
                if cfg!(windows) {
                    root.join("bin/llama/llama-cli.exe")
                } else {
                    root.join("bin/llama/llama-cli")
                }
            });

        Self {
            binary_path,
            model_path: root.join("models/weights").join(MODEL_FILE_NAME),
            thread_count: 4,
            gpu_layers: 99,
            context_size: 4096,
            max_tokens: 256,
            timeout: Duration::from_secs(240),
        }
    }
}

pub struct LlmEngine {
    config: LlmConfig,
}

impl LlmEngine {
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        if !config.binary_path.exists() {
            return Err(LlmError::BinaryNotFound(config.binary_path));
        }
        if !config.model_path.exists() {
            return Err(LlmError::ModelNotFound(config.model_path));
        }

        Ok(Self { config })
    }

    /// Verifies the SHA-256 hash of the quantized GGUF weights.
    pub fn verify_model_checksum(&self) -> Result<(), LlmError> {
        let mut file = File::open(&self.config.model_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 1024 * 1024]; // 1 MB buffer

        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let actual = format!("{:x}", hasher.finalize());
        if actual != EXPECTED_MODEL_SHA256 {
            return Err(LlmError::ChecksumMismatch {
                expected: EXPECTED_MODEL_SHA256.to_string(),
                actual,
            });
        }

        Ok(())
    }

    /// Runs local inference against the bundled Qwen3 model with greedy deterministic sampling.
    /// Optionally constrains output using a GBNF grammar string or file.
    pub fn generate(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        grammar: Option<&str>,
    ) -> Result<String, LlmError> {
        match self.generate_with(system_prompt, user_prompt, grammar, self.config.gpu_layers) {
            // A GPU run can fail on drivers or memory the build cannot use; the CPU path always works.
            Err(LlmError::ExecutionFailed { .. }) if self.config.gpu_layers > 0 => {
                self.generate_with(system_prompt, user_prompt, grammar, 0)
            }
            other => other,
        }
    }

    fn generate_with(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        grammar: Option<&str>,
        gpu_layers: usize,
    ) -> Result<String, LlmError> {
        let full_prompt = format!(
            // Qwen3 thinks before answering unless the assistant turn opens with an empty think block.
            "<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n",
            system_prompt.trim(),
            user_prompt.trim()
        );

        let bin_dir = self.config.binary_path.parent().unwrap();
        let mut cmd = Command::new(&self.config.binary_path);

        cmd.arg("-m")
            .arg(&self.config.model_path)
            .arg("-p")
            .arg(&full_prompt)
            .arg("-c")
            .arg(self.config.context_size.to_string())
            .arg("-t")
            .arg(self.config.thread_count.to_string())
            .arg("-n")
            .arg(self.config.max_tokens.to_string())
            .arg("-st") // Single-turn conversation
            .arg("--temp")
            .arg("0.0") // Strict deterministic greedy sampling
            .arg("--simple-io")
            .arg("-ngl")
            .arg(gpu_layers.to_string())
            .arg("--reasoning")
            .arg("off");

        // If grammar is specified, write to a temp grammar file
        let mut temp_grammar_path = None;
        if let Some(grammar_str) = grammar {
            let temp_path = std::env::temp_dir()
                .join(format!("saar_grammar_{}.gbnf", uuid::Uuid::new_v4()));
            std::fs::write(&temp_path, grammar_str)?;
            cmd.arg("--grammar-file").arg(&temp_path);
            temp_grammar_path = Some(temp_path);
        }

        // Configure shared library search path for bundled libllama.so / libggml.so
        let mut ld_paths = vec![bin_dir.to_str().unwrap().to_string()];
        if let Ok(existing_ld) = std::env::var("LD_LIBRARY_PATH") {
            ld_paths.push(existing_ld);
        }
        cmd.env("LD_LIBRARY_PATH", ld_paths.join(":"));

        let timeout = self.config.timeout;
        let handle = std::thread::spawn(move || cmd.output());

        let t0 = Instant::now();
        let output = loop {
            if handle.is_finished() {
                break handle.join().map_err(|_| LlmError::ExecutionFailed {
                    code: None,
                    stderr: "Thread join error during inference".to_string(),
                })??;
            }
            if t0.elapsed() > timeout {
                if let Some(gp) = temp_grammar_path {
                    let _ = std::fs::remove_file(gp);
                }
                return Err(LlmError::Timeout(timeout));
            }
            std::thread::sleep(Duration::from_millis(50));
        };

        if let Some(gp) = temp_grammar_path {
            let _ = std::fs::remove_file(gp);
        }

        if !output.status.success() {
            let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(LlmError::ExecutionFailed {
                code: output.status.code(),
                stderr: stderr_str,
            });
        }

        let raw_stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let answer = clean_llm_output(&raw_stdout).ok_or_else(|| LlmError::ExecutionFailed {
            code: output.status.code(),
            stderr: "Model output did not contain an assistant answer".to_string(),
        })?;
        // llama-cli reports some failures (for example a prompt larger than the context window)
        // as ordinary text with exit code 0. That is a failure, not an answer.
        if is_runtime_error_text(&answer) {
            return Err(LlmError::ExecutionFailed { code: output.status.code(), stderr: answer });
        }
        Ok(answer)
    }
}

/// True for text that llama-cli prints when it fails but still exits successfully.
fn is_runtime_error_text(answer: &str) -> bool {
    let head = answer.trim_start();
    head.starts_with("Error:") || head.starts_with("error:")
}

/// Extracts the model's answer from llama-cli stdout.
///
/// llama-cli prints a banner, echoes the prompt, then the answer, then timing and exit lines.
/// On Windows the console output uses CRLF, so line endings are normalized before matching.
/// Returns `None` when the assistant marker is missing, so the banner is never shown as an answer.
fn clean_llm_output(raw: &str) -> Option<String> {
    const ASSISTANT_MARKER: &str = "<|im_start|>assistant\n";
    const TRUNCATION_MARKER: &str = "(truncated)";
    let normalized = raw.replace("\r\n", "\n");

    // llama-cli echoes the prompt. For long prompts the echo is cut with "(truncated)" and the
    // answer starts on the following line, so the marker may be absent from the output.
    let mut out: &str = match normalized.rfind(ASSISTANT_MARKER) {
        Some(pos) => &normalized[pos + ASSISTANT_MARKER.len()..],
        None => {
            let pos = normalized.find(TRUNCATION_MARKER)?;
            let line_end = normalized[pos..].find('\n')? + pos + 1;
            &normalized[line_end..]
        }
    };

    if let Some(pos) = out.find("[ Prompt:") {
        out = &out[..pos];
    }
    if let Some(pos) = out.find("Exiting...") {
        out = &out[..pos];
    }
    let mut answer = out.trim_end_matches("<|im_end|>").trim();
    // The prompt ends with an empty think block; the echo or the model may repeat it.
    while let Some(rest) = answer.strip_prefix("<think>") {
        match rest.find("</think>") {
            Some(end) => answer = rest[end + "</think>".len()..].trim_start(),
            None => return None,
        }
    }
    Some(answer.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_paths_resolve() {
        let config = LlmConfig::default();
        assert!(
            config.binary_path.exists(),
            "llama-cli must exist at default path"
        );
        assert!(
            config.model_path.parent().map(|p| p.exists()).unwrap_or(false),
            "Model directory must exist at default path"
        );
    }

    #[test]
    fn test_clean_llm_output() {
        let raw = "<|im_start|>system\n...\n<|im_start|>assistant\nHello World!<|im_end|>";
        assert_eq!(clean_llm_output(raw).as_deref(), Some("Hello World!"));
    }

    #[test]
    fn test_clean_llm_output_strips_windows_banner_and_timing() {
        // Shape captured from llama-cli on Windows: CRLF line endings, banner, prompt echo, timing.
        let raw = "Loading model... \r\n\r\nbuild      : b11541\r\n\r\navailable commands:\r\n\r\n\
                   > <|im_start|>system\r\nYou answer briefly.<|im_end|>\r\n<|im_start|>user\r\nWhat is 2+2?<|im_end|>\r\n\
                   <|im_start|>assistant\r\n\r\n2+2 equals 4.\r\n\r\n[ Prompt: 239.6 t/s | Generation: 14.1 t/s ]\r\n\r\nExiting...\r\n";
        assert_eq!(clean_llm_output(raw).as_deref(), Some("2+2 equals 4."));
    }

    #[test]
    fn test_clean_llm_output_handles_truncated_prompt_echo() {
        // Long prompts are echoed truncated; the answer follows the truncated line.
        let raw = "available commands:\r\n\r\n> <|im_start|>system\r\nYou are SAAR. State amounts in format ... (truncated)\r\n\
                   Net figures: 742.81 USD [inv:abc].\r\n\r\n[ Prompt: 40.0 t/s | Generation: 9.0 t/s ]\r\n\r\nExiting...\r\n";
        assert_eq!(
            clean_llm_output(raw).as_deref(),
            Some("Net figures: 742.81 USD [inv:abc].")
        );
    }

    #[test]
    fn test_clean_llm_output_drops_the_empty_think_block() {
        let raw = "> <|im_start|>user\nHi<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\nHello there.\n\n[ Prompt: 1 t/s ]\n";
        assert_eq!(clean_llm_output(raw).as_deref(), Some("Hello there."));
    }

    #[test]
    fn test_runtime_error_text_is_not_an_answer() {
        assert!(is_runtime_error_text("Error: request (8311 tokens) exceeds the available context size (4096 tokens)"));
        assert!(is_runtime_error_text("  error: failed to load model"));
        assert!(!is_runtime_error_text("The error rate was 2%."));
    }

    #[test]
    fn test_clean_llm_output_rejects_output_without_answer_marker() {
        assert_eq!(clean_llm_output("Loading model...\r\navailable commands:\r\n"), None);
    }
}
