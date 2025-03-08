use axum::body::Bytes;
use futures::Stream;
use s3::{
    bucket::Bucket,
    error::S3Error,
    creds::{
        Credentials,
        error::CredentialsError
    },
    region::Region
};
use sha2::{
    Digest,
    Sha256
};
use std::{
    future::Future,
    io,
    path::Path
};
use thiserror::Error;
use tokio::io::{
    AsyncRead,
    AsyncWrite
};
use tokio_util::io::{
    InspectReader,
    StreamReader
};

#[derive(Debug, Error)]
pub enum UploadError {
    #[error("I/O error: {0}")]
    IOError(#[from] io::Error),
    #[error("Invalid filename")]
    InvalidFilename,
    #[error("Bucket error: {0}")]
    S3Error(#[from] S3Error),
}

pub fn require_filename(path: &str) -> Result<&str, UploadError> {
    let p = Path::new(path);

    if p.file_name().is_some() && p.components().count() == 1 {
        Ok(path)
    }
    else {
        Err(UploadError::InvalidFilename)
    }
}

pub async fn stream_to_writer<S, W>(
    stream: S,
    writer: W
) -> Result<(String, u64), UploadError>
where
    S: Stream<Item = Result<Bytes, io::Error>> + Send,
    W: AsyncWrite
{
    // make hashing reader
    let mut hasher = Sha256::new();
    let reader = InspectReader::new(
        StreamReader::new(stream),
        |buf| hasher.update(&buf)
    );

    // read stream
    futures::pin_mut!(reader);
    futures::pin_mut!(writer);
    let size = tokio::io::copy(&mut reader, &mut writer).await?;
    let sha256 = format!("{:x}", hasher.finalize());

    Ok((sha256, size))
}

pub trait Uploader {
    fn upload<R>(
        &self,
        _filename: &str,
        _reader: R
    ) -> impl Future<Output = Result<String, UploadError>> + Send
    where
        R: AsyncRead + Unpin + Send;
}

pub struct LocalUploader {
    pub uploads_directory: String
}

impl Uploader for LocalUploader {
    async fn upload<R>(
        &self,
        filename: &str,
        mut _reader: R
    ) -> Result<String, UploadError>
    where
        R: AsyncRead + Unpin + Send
    {
        Ok(
            format!(
                "http://localhost:3000/{0}/{filename}",
                self.uploads_directory
            )
        )
    }
}

#[derive(Debug, Error)]
pub enum BucketUploaderError {
    #[error("Bucket error: {0}")]
    S3Error(#[from] S3Error),
    #[error("Credentials error: {0}")]
    CredentialsError(#[from] CredentialsError)
}

pub struct BucketUploader {
    bucket: Bucket,
    base_url: String,
    base_dir: String
}

impl BucketUploader {
    pub fn new(
        name: &str,
        region: &str,
        endpoint: &str,
        access_key: &str,
        secret_key: &str,
        base_url: &str,
        base_dir: &str
    ) -> Result<BucketUploader, BucketUploaderError> {
        let mut bucket = *Bucket::new(
            name,
            Region::Custom {
                region: region.into(),
                endpoint: endpoint.into()
            },
            Credentials::new(
                Some(access_key),
                Some(secret_key),
                None,
                None,
                None
            )?
        )?;
        bucket.set_path_style();

        Ok(
            BucketUploader {
                bucket,
                base_url: base_url.into(),
                base_dir: base_dir.into()
            }
        )
    }
}

impl Uploader for BucketUploader {
    async fn upload<R>(
        &self,
        filename: &str,
        mut reader: R
    ) -> Result<String, UploadError>
    where
        R: AsyncRead + Unpin + Send
    {
        let path = format!("{0}/{filename}", self.base_dir);
// TODO: check return code?
        self.bucket.put_object_stream(&mut reader, &path).await?;
        Ok(format!("{0}/{path}", self.base_url))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn require_filename_empty() {
        assert!(
            matches!(
                require_filename("").unwrap_err(),
                UploadError::InvalidFilename
            )
        );
    }

    #[test]
    fn require_filename_dots() {
        assert!(
            matches!(
                require_filename("../bad").unwrap_err(),
                UploadError::InvalidFilename
            )
        );
    }

    #[test]
    fn require_filename_multiple_components() {
        assert!(
            matches!(
                require_filename("one/two/three").unwrap_err(),
                UploadError::InvalidFilename
            )
        );
    }

    #[test]
    fn require_filename_root() {
        assert!(
            matches!(
                require_filename("/").unwrap_err(),
                UploadError::InvalidFilename
            )
        );
    }

    #[test]
    fn require_filename_ok() {
        assert_eq!(
            require_filename("filename").unwrap(),
            "filename"
        );
    }
}
