//! Input limits for get-upload-url (S3 multipart max parts = 10_000).

pub const S3_MAX_PARTS: u64 = 10_000;

#[derive(Debug, Clone)]
pub struct UploadLimits {
    pub part_size: u64,
    pub max_upload_bytes: u64,
    pub max_title_chars: usize,
    pub allowed_content_types: Vec<String>,
    pub presign_ttl_secs: u64,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum UploadValidationError {
    #[error("file_size must be greater than zero")]
    EmptyFile,
    #[error("file_size {size} exceeds max_upload_bytes {max}")]
    TooLarge { size: u64, max: u64 },
    #[error("multipart would need {parts} parts (max {max})")]
    TooManyParts { parts: u64, max: u64 },
    #[error("title longer than {max} characters")]
    TitleTooLong { max: usize },
    #[error("content_type not allowed: {0}")]
    ContentTypeDenied(String),
}

pub fn validate_upload_request(
    file_size: u64,
    title: &Option<String>,
    content_type: &Option<String>,
    limits: &UploadLimits,
) -> Result<(), UploadValidationError> {
    if file_size == 0 {
        return Err(UploadValidationError::EmptyFile);
    }
    if file_size > limits.max_upload_bytes {
        return Err(UploadValidationError::TooLarge {
            size: file_size,
            max: limits.max_upload_bytes,
        });
    }
    let parts = (file_size + limits.part_size - 1) / limits.part_size.max(1);
    if parts > S3_MAX_PARTS {
        return Err(UploadValidationError::TooManyParts {
            parts,
            max: S3_MAX_PARTS,
        });
    }
    if let Some(t) = title {
        if t.chars().count() > limits.max_title_chars {
            return Err(UploadValidationError::TitleTooLong {
                max: limits.max_title_chars,
            });
        }
    }
    if let Some(ct) = content_type {
        let normalized = ct.trim().to_ascii_lowercase();
        let ok = limits
            .allowed_content_types
            .iter()
            .any(|allowed| normalized == allowed.as_str());
        if !ok {
            return Err(UploadValidationError::ContentTypeDenied(ct.clone()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> UploadLimits {
        UploadLimits {
            part_size: 16,
            max_upload_bytes: 1000,
            max_title_chars: 5,
            allowed_content_types: vec!["video/mp4".into()],
            presign_ttl_secs: 60,
        }
    }

    #[test]
    fn rejects_empty_and_oversize() {
        let l = limits();
        assert_eq!(
            validate_upload_request(0, &None, &None, &l),
            Err(UploadValidationError::EmptyFile)
        );
        assert!(matches!(
            validate_upload_request(1001, &None, &None, &l),
            Err(UploadValidationError::TooLarge { .. })
        ));
    }

    #[test]
    fn rejects_bad_content_type_and_title() {
        let l = limits();
        assert!(matches!(
            validate_upload_request(10, &None, &Some("text/plain".into()), &l),
            Err(UploadValidationError::ContentTypeDenied(_))
        ));
        assert!(matches!(
            validate_upload_request(10, &Some("abcdef".into()), &None, &l),
            Err(UploadValidationError::TitleTooLong { .. })
        ));
        assert!(validate_upload_request(10, &Some("hi".into()), &Some("video/mp4".into()), &l).is_ok());
    }
}
