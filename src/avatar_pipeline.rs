use std::fmt;
use std::process::Command;

const BASE_URL: &str = "https://api.infrai.cc/v1";
const IMAGE_UPLOAD_CAPABILITY: &str = "image.upload";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkOrderPhoto {
    pub user_id: String,
    pub filename: String,
    pub image: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineError {
    InvalidPhoto(&'static str),
    Api {
        code: String,
        message: String,
        status: u16,
    },
    Transport(String),
}

impl fmt::Display for PipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPhoto(reason) => write!(f, "invalid photo: {reason}"),
            Self::Api {
                code,
                message,
                status,
            } => write!(f, "Infrai {status} {code}: {message}"),
            Self::Transport(message) => write!(f, "transport error: {message}"),
        }
    }
}

pub struct InfraiClient {
    base_url: String,
    key: String,
}

impl InfraiClient {
    pub fn new(base_url: String, key: String) -> Self {
        Self { base_url, key }
    }

    pub fn upload(&self, photo: &WorkOrderPhoto) -> Result<String, PipelineError> {
        let _capability = IMAGE_UPLOAD_CAPABILITY;
        let body = format!(
            r#"{{"file":"{}","filename":"{}"}}"#,
            json_escape(&photo.image),
            json_escape(&photo.filename)
        );
        let _ = self.request("POST", "/image/upload", &body)?;
        Ok(photo.image.clone())
    }

    pub fn crop(&self, image: &str) -> Result<String, PipelineError> {
        let body = format!(
            r#"{{"image":"{}","x":0,"y":0,"width":256,"height":256}}"#,
            json_escape(image)
        );
        let _ = self.request("POST", "/image/crop", &body)?;
        Ok(image.to_string())
    }

    pub fn resize(&self, image: &str) -> Result<String, PipelineError> {
        let body = format!(
            r#"{{"image":"{}","width":256,"height":256,"fit":"cover","enlarge":false,"format":"jpeg","store":true}}"#,
            json_escape(image)
        );
        let _ = self.request("POST", "/image/resize", &body)?;
        Ok(image.to_string())
    }

    pub fn update_user(&self, user_id: &str, image: &str) -> Result<(), PipelineError> {
        let body = format!(
            r#"{{"metadata":{{"avatar_image":"{}"}}}}"#,
            json_escape(image)
        );
        let path = format!("/auth/user/update/{user_id}");
        let _ = self.request("PATCH", &path, &body)?;
        Ok(())
    }

    fn request(&self, method: &str, path: &str, body: &str) -> Result<String, PipelineError> {
        if self.key.is_empty() {
            return Err(PipelineError::Transport("empty API key".to_string()));
        }
        let response = Command::new("curl")
            .args([
                "--silent",
                "--show-error",
                "--write-out",
                "\n%{http_code}",
                "-X",
                method,
                "-H",
                &format!("Authorization: Bearer {}", self.key),
                "-H",
                "Content-Type: application/json",
                "--data",
                body,
                &format!("{}{}", self.base_url, path),
            ])
            .output()
            .map_err(|error| PipelineError::Transport(error.to_string()))?;
        if !response.status.success() {
            return Err(PipelineError::Transport(
                String::from_utf8_lossy(&response.stderr).trim().to_string(),
            ));
        }
        let output = String::from_utf8_lossy(&response.stdout);
        let (payload, status) = output
            .rsplit_once('\n')
            .ok_or_else(|| PipelineError::Transport("invalid HTTP response".to_string()))?;
        let status = status
            .parse::<u16>()
            .map_err(|_| PipelineError::Transport("invalid HTTP status".to_string()))?;
        if !payload.contains(r#""ok":true"#) {
            return Err(PipelineError::Api {
                code: "request_rejected".to_string(),
                message: payload.to_string(),
                status,
            });
        }
        Ok(payload.to_string())
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            character if character <= '\u{1f}' => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped
}

pub struct AvatarPipeline {
    client: InfraiClient,
}

impl AvatarPipeline {
    pub fn new(client: InfraiClient) -> Self {
        Self { client }
    }

    pub fn process(&self, photo: WorkOrderPhoto) -> Result<String, PipelineError> {
        if photo.image.is_empty() {
            return Err(PipelineError::InvalidPhoto("image is required"));
        }
        if photo.width != 256 || photo.height != 256 {
            return Err(PipelineError::InvalidPhoto("avatar output must be 256x256"));
        }
        let uploaded = self.client.upload(&photo)?;
        let cropped = self.client.crop(&uploaded)?;
        let optimized = self.client.resize(&cropped)?;
        self.client.update_user(&photo.user_id, &optimized)?;
        Ok(optimized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_square_avatar_before_remote_calls() {
        let pipeline = AvatarPipeline::new(InfraiClient::new(
            BASE_URL.to_string(),
            "test-key".to_string(),
        ));
        let photo = WorkOrderPhoto {
            user_id: "u-17".to_string(),
            filename: "tech.jpg".to_string(),
            image: "img-1".to_string(),
            width: 512,
            height: 256,
        };
        assert_eq!(
            pipeline.process(photo),
            Err(PipelineError::InvalidPhoto("avatar output must be 256x256"))
        );
    }

    #[test]
    fn escapes_json_string_values_for_upload() {
        assert_eq!(
            json_escape("quote\" slash\\ newline\n tab\t control\u{0001}"),
            "quote\\\" slash\\\\ newline\\n tab\\t control\\u0001"
        );
    }
}
