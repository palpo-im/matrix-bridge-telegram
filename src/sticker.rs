use std::process::Command;

use anyhow::Result;
use tracing::{debug, info, warn};

use crate::config::AnimatedStickerConfig;

/// Sticker format types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StickerFormat {
    /// Static WebP sticker
    Webp,
    /// Animated TGS (gzipped Lottie JSON)
    Tgs,
    /// Video sticker (WebM)
    Webm,
    /// Unknown format
    Unknown,
}

/// Handles conversion of Telegram stickers to Matrix-compatible formats.
pub struct StickerConverter {
    config: AnimatedStickerConfig,
    lottie_available: bool,
    ffmpeg_available: bool,
}

impl StickerConverter {
    pub fn new(config: AnimatedStickerConfig) -> Self {
        let lottie_available = Self::check_command("lottieconverter");
        let ffmpeg_available = Self::check_command("ffmpeg");

        if !lottie_available {
            info!("lottieconverter not found - TGS stickers will be sent as-is");
        }
        if !ffmpeg_available {
            info!("ffmpeg not found - WebM stickers will be sent as-is");
        }

        Self {
            config,
            lottie_available,
            ffmpeg_available,
        }
    }

    /// Check if a command-line tool is available.
    fn check_command(name: &str) -> bool {
        Command::new(name)
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Detect sticker format from filename or MIME type.
    pub fn detect_format(filename: &str, mime_type: Option<&str>) -> StickerFormat {
        let lower = filename.to_lowercase();
        if lower.ends_with(".tgs") {
            return StickerFormat::Tgs;
        }
        if lower.ends_with(".webm") {
            return StickerFormat::Webm;
        }
        if lower.ends_with(".webp") {
            return StickerFormat::Webp;
        }

        match mime_type {
            Some("application/x-tgsticker") => StickerFormat::Tgs,
            Some("video/webm") => StickerFormat::Webm,
            Some("image/webp") => StickerFormat::Webp,
            _ => StickerFormat::Unknown,
        }
    }

    /// Convert a sticker to a Matrix-compatible format.
    /// Returns (converted_data, new_mime_type, new_filename).
    pub async fn convert(
        &self,
        data: &[u8],
        format: StickerFormat,
    ) -> Result<(Vec<u8>, String, String)> {
        match format {
            StickerFormat::Tgs => self.convert_tgs(data).await,
            StickerFormat::Webm => self.convert_webm(data).await,
            StickerFormat::Webp => {
                // WebP is supported by Matrix clients, pass through
                Ok((data.to_vec(), "image/webp".to_string(), "sticker.webp".to_string()))
            }
            StickerFormat::Unknown => {
                Ok((data.to_vec(), "application/octet-stream".to_string(), "sticker".to_string()))
            }
        }
    }

    /// Convert TGS (gzipped Lottie) to PNG or GIF.
    async fn convert_tgs(&self, data: &[u8]) -> Result<(Vec<u8>, String, String)> {
        if !self.lottie_available {
            debug!("lottieconverter not available, returning TGS as-is");
            return Ok((
                data.to_vec(),
                "application/x-tgsticker".to_string(),
                "sticker.tgs".to_string(),
            ));
        }

        let target = &self.config.target;
        let width = self.config.args.width;
        let height = self.config.args.height;
        let fps = self.config.args.fps;

        // Write input to temp file
        let input_path = std::env::temp_dir().join(format!("tgs_input_{}.tgs", uuid::Uuid::new_v4()));
        let output_ext = if target == "gif" { "gif" } else { "png" };
        let output_path = std::env::temp_dir().join(format!(
            "tgs_output_{}.{}",
            uuid::Uuid::new_v4(),
            output_ext
        ));

        tokio::fs::write(&input_path, data).await?;

        let result = tokio::task::spawn_blocking({
            let input = input_path.clone();
            let output = output_path.clone();
            let target = target.clone();
            move || {
                let mut cmd = Command::new("lottieconverter");
                cmd.arg(&input)
                    .arg(&output)
                    .arg(&target)
                    .arg(format!("{}x{}", width, height));

                if target == "gif" {
                    cmd.arg(fps.to_string());
                }

                let cmd_output = cmd.output()?;
                if !cmd_output.status.success() {
                    let stderr = String::from_utf8_lossy(&cmd_output.stderr);
                    anyhow::bail!("lottieconverter failed: {}", stderr);
                }
                Ok(())
            }
        })
        .await?;

        // Clean up input
        let _ = tokio::fs::remove_file(&input_path).await;

        match result {
            Ok(()) => {
                let converted = tokio::fs::read(&output_path).await?;
                let _ = tokio::fs::remove_file(&output_path).await;

                let (mime, filename) = if target == "gif" {
                    ("image/gif".to_string(), "sticker.gif".to_string())
                } else {
                    ("image/png".to_string(), "sticker.png".to_string())
                };

                info!(
                    "Converted TGS sticker: {} bytes -> {} bytes ({})",
                    data.len(),
                    converted.len(),
                    mime
                );

                Ok((converted, mime, filename))
            }
            Err(e) => {
                let _ = tokio::fs::remove_file(&output_path).await;
                warn!("TGS conversion failed, returning original: {}", e);
                Ok((
                    data.to_vec(),
                    "application/x-tgsticker".to_string(),
                    "sticker.tgs".to_string(),
                ))
            }
        }
    }

    /// Convert WebM video sticker to GIF or MP4.
    async fn convert_webm(&self, data: &[u8]) -> Result<(Vec<u8>, String, String)> {
        if !self.ffmpeg_available || !self.config.convert_from_webm {
            debug!("WebM conversion not available or disabled, returning as-is");
            return Ok((
                data.to_vec(),
                "video/webm".to_string(),
                "sticker.webm".to_string(),
            ));
        }

        let width = self.config.args.width;
        let height = self.config.args.height;
        let target = &self.config.target;

        let input_path = std::env::temp_dir().join(format!("webm_input_{}.webm", uuid::Uuid::new_v4()));
        let output_ext = if target == "gif" { "gif" } else { "mp4" };
        let output_path = std::env::temp_dir().join(format!(
            "webm_output_{}.{}",
            uuid::Uuid::new_v4(),
            output_ext
        ));

        tokio::fs::write(&input_path, data).await?;

        let result = tokio::task::spawn_blocking({
            let input = input_path.clone();
            let output = output_path.clone();
            let target = target.clone();
            move || {
                let mut cmd = Command::new("ffmpeg");
                cmd.arg("-i").arg(&input)
                    .arg("-y") // overwrite
                    .arg("-v").arg("quiet");

                if target == "gif" {
                    cmd.arg("-vf")
                        .arg(format!(
                            "scale={}:{}:force_original_aspect_ratio=decrease",
                            width, height
                        ))
                        .arg("-loop").arg("0");
                } else {
                    cmd.arg("-vf")
                        .arg(format!(
                            "scale={}:{}:force_original_aspect_ratio=decrease",
                            width, height
                        ))
                        .arg("-c:v").arg("libx264")
                        .arg("-pix_fmt").arg("yuv420p")
                        .arg("-movflags").arg("+faststart");
                }

                cmd.arg(&output);

                let cmd_output = cmd.output()?;
                if !cmd_output.status.success() {
                    let stderr = String::from_utf8_lossy(&cmd_output.stderr);
                    anyhow::bail!("ffmpeg failed: {}", stderr);
                }
                Ok(())
            }
        })
        .await?;

        let _ = tokio::fs::remove_file(&input_path).await;

        match result {
            Ok(()) => {
                let converted = tokio::fs::read(&output_path).await?;
                let _ = tokio::fs::remove_file(&output_path).await;

                let (mime, filename) = if target == "gif" {
                    ("image/gif".to_string(), "sticker.gif".to_string())
                } else {
                    ("video/mp4".to_string(), "sticker.mp4".to_string())
                };

                info!(
                    "Converted WebM sticker: {} bytes -> {} bytes ({})",
                    data.len(),
                    converted.len(),
                    mime
                );

                Ok((converted, mime, filename))
            }
            Err(e) => {
                let _ = tokio::fs::remove_file(&output_path).await;
                warn!("WebM conversion failed, returning original: {}", e);
                Ok((
                    data.to_vec(),
                    "video/webm".to_string(),
                    "sticker.webm".to_string(),
                ))
            }
        }
    }

    /// Check if conversion tools are available.
    pub fn capabilities(&self) -> StickerCapabilities {
        StickerCapabilities {
            tgs_to_png: self.lottie_available,
            tgs_to_gif: self.lottie_available,
            webm_to_gif: self.ffmpeg_available && self.config.convert_from_webm,
            webm_to_mp4: self.ffmpeg_available && self.config.convert_from_webm,
        }
    }
}

/// Reports which sticker conversions are available.
#[derive(Debug, Clone)]
pub struct StickerCapabilities {
    pub tgs_to_png: bool,
    pub tgs_to_gif: bool,
    pub webm_to_gif: bool,
    pub webm_to_mp4: bool,
}

impl Default for StickerConverter {
    fn default() -> Self {
        Self::new(AnimatedStickerConfig::default())
    }
}
