//! Multi-modal memory support
//! 
//! Supports storing embeddings for different content types:
//! - Text (default)
//! - Images
//! - Audio
//! - Video
//! - Documents (PDF, etc.)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryContentType {
    Text,
    Image,
    Audio,
    Video,
    Document,
}

impl MemoryContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Image => "image",
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Document => "document",
        }
    }

    pub fn from_str_value(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "image" => Self::Image,
            "audio" => Self::Audio,
            "video" => Self::Video,
            "document" | "pdf" | "doc" => Self::Document,
            _ => Self::Text,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalMemory {
    pub id: String,
    pub content_type: MemoryContentType,
    pub content: String,  // For text, or file path for others
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub thumbnail: Option<Vec<u8>>,  // For images/videos
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
    pub channels: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub duration_secs: Option<f64>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub duration_secs: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f64>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub page_count: Option<u32>,
    pub word_count: Option<u32>,
    pub format: Option<String>,
}

pub struct MultimodalProcessor;

impl MultimodalProcessor {
    /// Process an image file and extract metadata
    pub fn process_image(path: &Path) -> Result<ImageMetadata> {
        // For now, return basic metadata
        // In production, use image crate to read actual dimensions
        Ok(ImageMetadata {
            width: None,
            height: None,
            format: path.extension().and_then(|s| s.to_str()).map(String::from),
            channels: Some(3),  // Assume RGB
        })
    }

    /// Process an audio file
    pub fn process_audio(path: &Path) -> Result<AudioMetadata> {
        Ok(AudioMetadata {
            duration_secs: None,
            sample_rate: None,
            channels: None,
            format: path.extension().and_then(|s| s.to_str()).map(String::from),
        })
    }

    /// Process a video file
    pub fn process_video(path: &Path) -> Result<VideoMetadata> {
        Ok(VideoMetadata {
            duration_secs: None,
            width: None,
            height: None,
            fps: None,
            format: path.extension().and_then(|s| s.to_str()).map(String::from),
        })
    }

    /// Process a document
    pub fn process_document(path: &Path) -> Result<DocumentMetadata> {
        Ok(DocumentMetadata {
            page_count: None,
            word_count: None,
            format: path.extension().and_then(|s| s.to_str()).map(String::from),
        })
    }

    /// Supported file extensions
    pub fn supported_extensions(content_type: &MemoryContentType) -> Vec<&'static str> {
        match content_type {
            MemoryContentType::Image => vec!["jpg", "jpeg", "png", "gif", "webp", "bmp", "svg"],
            MemoryContentType::Audio => vec!["mp3", "wav", "ogg", "flac", "m4a", "aac"],
            MemoryContentType::Video => vec!["mp4", "webm", "avi", "mov", "mkv"],
            MemoryContentType::Document => vec!["pdf", "doc", "docx", "txt", "md", "rtf"],
            MemoryContentType::Text => vec!["txt", "md", "json", "xml", "csv"],
        }
    }

    pub fn detect_content_type(path: &Path) -> MemoryContentType {
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "svg" => MemoryContentType::Image,
            "mp3" | "wav" | "ogg" | "flac" | "m4a" | "aac" => MemoryContentType::Audio,
            "mp4" | "webm" | "avi" | "mov" | "mkv" => MemoryContentType::Video,
            "pdf" | "doc" | "docx" | "rtf" => MemoryContentType::Document,
            _ => MemoryContentType::Text,
        }
    }
}
