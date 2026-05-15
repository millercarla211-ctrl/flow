//! GLM-OCR integration for document OCR using vision models

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

const MODEL_FILE: &str = "GLM-OCR.Q4_K_M.gguf";
const MMPROJ_FILE: &str = "GLM-OCR.mmproj-Q8_0.gguf";
const MODEL_PATH: &str = "models/ocr/GLM-OCR.Q4_K_M.gguf";
const MMPROJ_PATH: &str = "models/ocr/GLM-OCR.mmproj-Q8_0.gguf";

pub struct GlmOcr {
    model_path: String,
    mmproj_path: String,
}

impl GlmOcr {
    pub fn required_model_paths() -> [&'static str; 2] {
        [MODEL_PATH, MMPROJ_PATH]
    }

    pub fn resolved_model_paths() -> [PathBuf; 2] {
        [
            resolve_model_path(MODEL_FILE),
            resolve_model_path(MMPROJ_FILE),
        ]
    }

    pub fn is_available() -> bool {
        Self::resolved_model_paths()
            .iter()
            .all(|path| path.exists())
    }

    pub fn new() -> Result<Self> {
        let [model_path, mmproj_path] = Self::resolved_model_paths();

        // Verify model files exist
        if !model_path.exists() {
            anyhow::bail!(
                "GLM-OCR model not found. Checked: {}",
                candidate_paths(MODEL_FILE)
                    .iter()
                    .map(|path| path.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        if !mmproj_path.exists() {
            anyhow::bail!(
                "GLM-OCR mmproj not found. Checked: {}",
                candidate_paths(MMPROJ_FILE)
                    .iter()
                    .map(|path| path.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        Ok(Self {
            model_path: model_path.to_string_lossy().into_owned(),
            mmproj_path: mmproj_path.to_string_lossy().into_owned(),
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

fn resolve_model_path(file_name: &str) -> PathBuf {
    candidate_paths(file_name)
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| PathBuf::from("models").join("ocr").join(file_name))
}

fn candidate_paths(file_name: &str) -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("models").join("ocr").join(file_name),
        PathBuf::from("models")
            .join("ocr")
            .join("glm-ocr-gguf")
            .join(file_name),
    ];

    if let Ok(root) = std::env::var("FLOW_MODEL_ROOT") {
        paths.push(PathBuf::from(&root).join("ocr").join(file_name));
        paths.push(
            PathBuf::from(root)
                .join("ocr")
                .join("glm-ocr-gguf")
                .join(file_name),
        );
    }

    #[cfg(windows)]
    paths.push(
        PathBuf::from(r"G:\Flow\data\models")
            .join("ocr")
            .join("glm-ocr-gguf")
            .join(file_name),
    );

    paths
}
