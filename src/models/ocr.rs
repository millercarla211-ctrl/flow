//! GLM-OCR integration for document OCR using vision models

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

const MODEL_PATH: &str = r"F:\flow\models\ocr\GLM-OCR.Q4_K_M.gguf";
const MMPROJ_PATH: &str = r"F:\flow\models\ocr\GLM-OCR.mmproj-Q8_0.gguf";

pub struct GlmOcr {
    model_path: String,
    mmproj_path: String,
}

impl GlmOcr {
    pub fn new() -> Result<Self> {
        // Verify model files exist
        if !Path::new(MODEL_PATH).exists() {
            anyhow::bail!(
                "GLM-OCR model not found at: {}\nRun: python scripts/download_glm_ocr_simple.py",
                MODEL_PATH
            );
        }

        if !Path::new(MMPROJ_PATH).exists() {
            anyhow::bail!(
                "GLM-OCR mmproj not found at: {}\nRun: python scripts/download_glm_ocr_simple.py",
                MMPROJ_PATH
            );
        }

        Ok(Self {
            model_path: MODEL_PATH.to_string(),
            mmproj_path: MMPROJ_PATH.to_string(),
        })
    }

    /// Perform OCR on an image using Python + llama-cpp-python
    pub fn ocr_image(&self, image_path: &str) -> Result<String> {
        // Check if image exists
        if !Path::new(image_path).exists() {
            anyhow::bail!("Image not found: {}", image_path);
        }

        println!("[OCR] Processing image: {}", image_path);
        println!("[OCR] Model: {}", self.model_path);
        println!("[OCR] MMProj: {}", self.mmproj_path);

        // Create a temporary Python script to run OCR
        let python_script = format!(
            r#"
import sys
try:
    from llama_cpp import Llama
    from llama_cpp.llama_chat_format import Llava15ChatHandler
except ImportError:
    print("ERROR: llama-cpp-python not installed")
    print("Install with: pip install llama-cpp-python")
    sys.exit(1)

# Initialize model with vision support
chat_handler = Llava15ChatHandler(clip_model_path=r"{}")
llm = Llama(
    model_path=r"{}",
    chat_handler=chat_handler,
    n_ctx=2048,
    n_gpu_layers=-1,
    verbose=False
)

# Perform OCR
messages = [
    {{
        "role": "user",
        "content": [
            {{"type": "image_url", "image_url": {{"url": r"{}"}}}},
            {{"type": "text", "text": "Extract all text from this image. Provide only the text content, no explanations."}}
        ]
    }}
]

response = llm.create_chat_completion(messages=messages)
print(response["choices"][0]["message"]["content"])
"#,
            self.mmproj_path, self.model_path, image_path
        );

        // Write script to temp file
        let script_path = "temp_ocr_script.py";
        std::fs::write(script_path, python_script)
            .context("Failed to write temporary Python script")?;

        // Execute Python script
        let output = Command::new("python")
            .arg(script_path)
            .output()
            .context("Failed to execute Python OCR script")?;

        // Clean up temp file
        let _ = std::fs::remove_file(script_path);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("OCR failed: {}", stderr);
        }

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(result)
    }

    /// Perform OCR with a custom prompt
    pub fn ocr_with_prompt(&self, image_path: &str, prompt: &str) -> Result<String> {
        if !Path::new(image_path).exists() {
            anyhow::bail!("Image not found: {}", image_path);
        }

        println!("[OCR] Processing image: {}", image_path);
        println!("[OCR] Custom prompt: {}", prompt);

        let python_script = format!(
            r#"
import sys
try:
    from llama_cpp import Llama
    from llama_cpp.llama_chat_format import Llava15ChatHandler
except ImportError:
    print("ERROR: llama-cpp-python not installed")
    print("Install with: pip install llama-cpp-python")
    sys.exit(1)

chat_handler = Llava15ChatHandler(clip_model_path=r"{}")
llm = Llama(
    model_path=r"{}",
    chat_handler=chat_handler,
    n_ctx=2048,
    n_gpu_layers=-1,
    verbose=False
)

messages = [
    {{
        "role": "user",
        "content": [
            {{"type": "image_url", "image_url": {{"url": r"{}"}}}},
            {{"type": "text", "text": r"{}"}}
        ]
    }}
]

response = llm.create_chat_completion(messages=messages)
print(response["choices"][0]["message"]["content"])
"#,
            self.mmproj_path, self.model_path, image_path, prompt
        );

        let script_path = "temp_ocr_script.py";
        std::fs::write(script_path, python_script)
            .context("Failed to write temporary Python script")?;

        let output = Command::new("python")
            .arg(script_path)
            .output()
            .context("Failed to execute Python OCR script")?;

        let _ = std::fs::remove_file(script_path);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("OCR failed: {}", stderr);
        }

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(result)
    }
}

impl Default for GlmOcr {
    fn default() -> Self {
        Self::new().expect("Failed to initialize GLM-OCR")
    }
}
