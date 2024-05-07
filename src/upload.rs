use axum::{
    async_trait,
    body::Bytes
};
use futures::Stream;
use std::{
    io,
    path::Path
};
use thiserror::Error;
use tokio::{
    fs::File,
    io::{
        AsyncWrite,
        BufWriter
    }
};
use tokio_util::io::StreamReader;

#[derive(Debug, Error)]
pub enum UploadError {
    #[error("I/O error")]
    IOError(#[from] io::Error),
    #[error("Invalid filename")]
    InvalidFilename
}

fn require_filename(path: &str) -> Result<&str, UploadError> {
    let p = Path::new(path);

    if p.file_name().is_some() && p.components().count() == 1 {
        Ok(path)
    }
    else {
        Err(UploadError::InvalidFilename)
    }
}

pub async fn stream_to_file<S>(
    uploads_directory: &str,
    path: &str,
    stream: S
) -> Result<(), UploadError>
where
    S: Stream<Item = Result<Bytes, io::Error>>,
{
    let filename = require_filename(path)?;
    let path = std::path::Path::new(uploads_directory).join(filename);
    let file = BufWriter::new(File::create(path).await?);
//    let mut file = tokio::io::sink();

    stream_to_writer(stream, file).await
}

pub async fn stream_to_writer<S, W>(
    stream: S,
    writer: W
) -> Result<(), UploadError>
where
    S: Stream<Item = Result<Bytes, io::Error>>,
    W: AsyncWrite
{
    let reader = StreamReader::new(stream);

    futures::pin_mut!(reader);
    futures::pin_mut!(writer);

    tokio::io::copy(&mut reader, &mut writer).await?;

    Ok(())
}

#[async_trait]
pub trait Uploader {
    async fn upload<S>(
        &self,
        _filename: &str,
        _stream: S
    ) -> Result<String, UploadError>
    where
        S: Stream<Item = Result<Bytes, io::Error>> + Send;
}

pub struct LocalUploader {
    pub uploads_directory: String
}

#[async_trait]
impl Uploader for LocalUploader {
    async fn upload<S>(
        &self,
        filename: &str,
        stream: S
    ) -> Result<String, UploadError>
    where
        S: Stream<Item = Result<Bytes, io::Error>> + Send
    {
        stream_to_file("uploads", filename, stream).await?;

        Ok(format!("http://localhost:3000/uploads/{filename}"))
    }
}
