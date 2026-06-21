use async_tempfile::TempFile;
use axum::{
    body::Bytes,
    extract::{Path, Request, State},
    response::{Json, Redirect}
};
use axum_extra::{
    extract::Query as MultiQuery,
    TypedHeader,
    headers::{ContentLength, ContentType}
};
use futures::{Stream, TryStreamExt};
use glc::{
    discourse::UserUpdatePost,
    model::{Flags, ProjectData, Projects, Publishers, Tags, Users}
};
use hex;
use http_body_util::{BodyExt, Limited, LengthLimitError};
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    io
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter};
use tokio_util::io::{InspectWriter, StreamReader};
use tracing::info;

use crate::{
    core::CoreArc,
    errors::AppError,
    extractors::{DiscourseEvent, ProjectPackage, ProjectPackageRelease, Wrapper},
    input::{FlagPost, GalleryPatch, PackageDataPatch, PackageDataPost, ProjectDataPatch, ProjectDataPost},
    model::{Admin, Flag, Owned, Project, User},
    params::ProjectsParams,
    upload::safe_filename
};

pub async fn not_found() -> Result<(), AppError>
{
    Err(AppError::NotFound)
}

pub async fn forbidden() -> Result<(), AppError>
{
    Err(AppError::Forbidden)
}

pub async fn root_get() -> &'static str {
    "hello world"
}

pub async fn projects_get(
    Wrapper(MultiQuery(params)): Wrapper<MultiQuery<ProjectsParams>>,
    State(core): State<CoreArc>
) -> Result<Json<Projects>, AppError>
{
    Ok(Json(core.get_projects(params).await?))
}

pub async fn project_get(
    proj: Project,
    State(core): State<CoreArc>
) -> Result<Json<ProjectData>, AppError>
{
    Ok(Json(core.get_project(proj).await?))
}

pub async fn project_post(
    owner: User,
    Path(proj): Path<String>,
    State(core): State<CoreArc>,
    Wrapper(Json(proj_data)): Wrapper<Json<ProjectDataPost>>
) -> Result<(), AppError>
{
    Ok(core.create_project(owner, &proj, &proj_data).await?)
}

pub async fn project_patch(
    Owned(owner, proj): Owned,
    State(core): State<CoreArc>,
    Wrapper(Json(proj_data)): Wrapper<Json<ProjectDataPatch>>
) -> Result<(), AppError>
{
    Ok(core.update_project(owner, proj, &proj_data).await?)
}

pub async fn project_revision_get(
    proj: Project,
    Path((_, revision)): Path<(String, u32)>,
    State(core): State<CoreArc>
) -> Result<Json<ProjectData>, AppError>
{
    Ok(Json(core.get_project_revision(proj, revision as i64).await?))
}

pub async fn owners_get(
    proj: Project,
    State(core): State<CoreArc>
) -> Result<Json<Users>, AppError>
{
    Ok(Json(core.get_owners(proj).await?))
}

pub async fn owners_add(
    Owned(_, proj): Owned,
    State(core): State<CoreArc>,
    Wrapper(Json(owners)): Wrapper<Json<Users>>
) -> Result<(), AppError>
{
    Ok(core.add_owners(&owners, proj).await?)
}

pub async fn owners_remove(
    Owned(_, proj): Owned,
    State(core): State<CoreArc>,
    Wrapper(Json(owners)): Wrapper<Json<Users>>
) -> Result<(), AppError>
{
    Ok(core.remove_owners(&owners, proj).await?)
}

pub async fn players_get(
    proj: Project,
    State(core): State<CoreArc>
) -> Result<Json<Users>, AppError>
{
    Ok(Json(core.get_players(proj).await?))
}

pub async fn players_add(
    requester: User,
    proj: Project,
    State(core): State<CoreArc>
) -> Result<(), AppError>
{
    Ok(core.add_player(requester, proj).await?)
}

pub async fn players_remove(
    requester: User,
    proj: Project,
    State(core): State<CoreArc>,
) -> Result<(), AppError>
{
    Ok(core.remove_player(requester, proj).await?)
}

pub async fn package_post(
    Owned(owner, proj): Owned,
    Path((_, pkg)): Path<(String, String)>,
    State(core): State<CoreArc>,
    Wrapper(Json(pkg_data)): Wrapper<Json<PackageDataPost>>
) -> Result<(), AppError>
{
    Ok(core.create_package(owner, proj, &pkg, &pkg_data).await?)
}

pub async fn package_patch(
    Owned(owner, proj): Owned,
    ProjectPackage(_, pkg): ProjectPackage,
    State(core): State<CoreArc>,
    Wrapper(Json(pkg_data)): Wrapper<Json<PackageDataPatch>>
) -> Result<(), AppError>
{
    Ok(core.update_package(owner, proj, pkg, &pkg_data).await?)
}

pub async fn package_delete(
    Owned(owner, proj): Owned,
    ProjectPackage(_, pkg): ProjectPackage,
    State(core): State<CoreArc>
) -> Result<(), AppError>
{
    Ok(core.delete_package(owner, proj, pkg).await?)
}

pub async fn release_post(
    Owned(owner, proj): Owned,
    ProjectPackage(_, pkg): ProjectPackage,
    Path((_, _, version)): Path<(String, String, String)>,
    State(core): State<CoreArc>
) -> Result<(), AppError>
{
    Ok(core.create_release(owner, proj, pkg, &version).await?)
}

pub async fn release_delete(
    Owned(owner, proj): Owned,
    ProjectPackageRelease(_, _, release): ProjectPackageRelease,
    State(core): State<CoreArc>
) -> Result<(), AppError>
{
    Ok(core.delete_release(owner, proj, release).await?)
}

fn unpack_limited_error(e: Box<dyn Error + Sync + Send>) -> io::Error {
    // turn boxed error back into io::Error
    match e.downcast::<io::Error>() {
        Ok(e) => *e,
        Err(e) => match e.downcast::<LengthLimitError>() {
            Ok(e) => io::Error::new(io::ErrorKind::FileTooLarge, e),
            Err(e) => io::Error::other(e)
        }
    }
}

fn into_limited_stream(
    request: Request,
    limit: usize
) -> Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send + Unpin>
{
     Box::new(
        Limited::new(request.into_body(), limit)
            .into_data_stream()
            .map_err(unpack_limited_error)
    )
}

/*
  TODO: Sending a 413 repsonse before reading all the data causes clients
  using HTTP/1.1 to ignore the response and keep the connection open, waiting
  for a response until the gateway in front of the GLS times out and returns
  a 502. It would be more robust to wait until the client is done sending
  data if we're using HTTP/1.1.
*/

fn limit_content_length(
    content_length: Option<u64>,
    max_size: usize
) -> Result<usize, AppError>
{
    if let Some(content_length) = content_length {
        match usize::try_from(content_length) {
            Err(_) => Err(AppError::TooLarge),
            Ok(cl) if cl > max_size => Err(AppError::TooLarge),
            Ok(cl) => Ok(cl)
        }
    }
    else {
        Ok(max_size)
    }
}

/*
async fn write_file<S, F>(
    stream: S,
    file: F
) ->  Result<(String, u64), io::Error>
where
    S: Stream<Item = Result<Bytes, io::Error>>  + Unpin,
    F: AsyncWrite + Unpin
{
    let mut off = 0;
    let mut reader = BufReader::new(StreamReader::new(stream));

    // make hashing writer
    let mut hasher = Sha256::new();
    let mut writer = BufWriter::new(
        InspectWriter::new(
            file,
            |buf| {
                hasher.update(buf);
                off += buf.len();
                info!("{off} {}", buf.len());
            }
        )
    );

    // read stream
    let size = tokio::io::copy(&mut reader, &mut writer).await?;
    let sha256 = format!("{}", hex::encode(hasher.finalize()));

    Ok((sha256, size))
}
*/

/*
async fn copy_stream_to_writer<S, F>(
    mut stream: S,
    file: F
) ->  Result<u64, io::Error>
where
    S: Stream<Item = Result<Bytes, io::Error>>  + Unpin,
    F: AsyncWrite + Unpin
{
    let mut off = 0;

    use futures::StreamExt;
    while let Some(r) = stream.next().await {
        match r {
            Err(e) => return Err(e),
            Ok(buf) => { off += buf.len(); }
        }
    }

    Ok(off as u64)
}
*/

/*
async fn copy_stream_to_writer<S, W>(
    stream: S,
    mut writer: W
) ->  Result<(String, u64), io::Error>
where
    S: Stream<Item = Result<Bytes, io::Error>> + Send + Unpin + 'static,
    W: AsyncWrite + Send + Unpin + 'static
{
    let (r_tx, mut r_rx) = tokio::sync::mpsc::channel(1);
    let (w_tx, mut w_rx) = tokio::sync::mpsc::channel(1);

    let rfut: tokio::task::JoinHandle<Result<_, io::Error>> = tokio::spawn(async move {
        let mut reader = StreamReader::new(stream);
        let mut buf = Vec::with_capacity(32768);

        loop {
            buf.clear();

            match reader.read_buf(&mut buf).await? {
                0 => break,
                _ => { w_tx.send(buf).await.map_err(io::Error::other)?; }
            };

            buf = match r_rx.recv().await {
                Some(buf) => buf,
                None => break
            };
        }

        Ok(())
    });

    let wfut: tokio::task::JoinHandle<Result<_, io::Error>> = tokio::spawn(async move {
        let mut hasher = Sha256::new();
        let mut off = 0;

        let mut buf = Vec::with_capacity(32768);

        loop {
            // we don't care if this is received; it won't be in the
            // case where the input stream is empty
            let _ = r_tx.send(buf).await;

            buf = match w_rx.recv().await {
                Some(buf) => buf,
                None => break
            };

            hasher.update(&buf[..]);
            writer.write_all(&buf[..]).await?;
            off += buf.len() as u64;
            info!("{off} {}", buf.len());
        }

        let sha256 = format!("{}", hex::encode(hasher.finalize()));

        Ok((sha256, off))
    });

    rfut.await??;
//    wfut.await?
    let (sha256, off) = wfut.await??;
    Ok((sha256, off))
}
*/

/*
async fn copy_stream_to_writer<S, W>(
    mut stream: S,
    mut writer: W
) ->  Result<(String, u64), io::Error>
where
    S: Stream<Item = Result<Bytes, io::Error>> + Send + Unpin + 'static,
    W: AsyncWrite + Send + Unpin + 'static
{
    let (w_tx, mut w_rx) = tokio::sync::mpsc::channel(1);
//    let (w_tx, mut w_rx) = tokio::sync::mpsc::unbounded_channel();

    let rfut: tokio::task::JoinHandle<Result<_, io::Error>> = tokio::spawn(async move {
        use futures::StreamExt;
        while let Some(r) = stream.next().await {
            match r {
                Err(e) => return Err(e),
                Ok(buf) => { w_tx.send(buf).await.map_err(io::Error::other)?; }
//                Ok(buf) => { w_tx.send(buf).map_err(io::Error::other)?; }
            }
        }

        Ok(())
    });

    let wfut: tokio::task::JoinHandle<Result<_, io::Error>> = tokio::spawn(async move {
        let mut hasher = Sha256::new();
        let mut off = 0;

        loop {
            let buf = match w_rx.recv().await {
                Some(buf) => buf,
                None => break
            };

            hasher.update(&buf[..]);
            writer.write_all(&buf[..]).await?;
            off += buf.len() as u64;
            info!("{off} {}", buf.len());
        }

        let sha256 = format!("{:x}", hasher.finalize());

        Ok((sha256, off))
    });

    rfut.await??;
    let (sha256, off) = wfut.await??;
    Ok((sha256, off))
}
*/

async fn copy_stream_to_writer<S, W>(
    stream: S,
    mut writer: W
) ->  Result<(String, u64), io::Error>
where
    S: Stream<Item = Result<Bytes, io::Error>> + Send + Unpin + 'static,
    W: AsyncWrite + Send + Unpin + 'static
{
    use std::sync::Arc;

    let (rw_tx, mut rw_rx) = tokio::sync::mpsc::channel(1);
    let (rh_tx, mut rh_rx) = tokio::sync::mpsc::channel(1);
    let (wr_tx, mut wr_rx) = tokio::sync::mpsc::channel(1);
    let (hr_tx, mut hr_rx) = tokio::sync::mpsc::channel(1);

    let rfut: tokio::task::JoinHandle<Result<_, io::Error>> = tokio::spawn(async move {
        let mut reader = StreamReader::new(stream);
        let mut buf = Vec::with_capacity(32768);

        loop {
            buf.clear();

            match reader.read_buf(&mut buf).await? {
                0 => break,
                _ => {
                    let buf = Arc::new(buf);
                    wr_tx.send(buf.clone()).await.map_err(io::Error::other)?;
                    hr_tx.send(buf).await.map_err(io::Error::other)?;
                }
            };

            let _ = rh_rx.recv().await;
            buf = match rw_rx.recv().await {
                Some(buf) => Arc::into_inner(buf).unwrap(),
                None => break
            };
        }

        Ok(())
    });

    let hfut: tokio::task::JoinHandle<Result<_, io::Error>> = tokio::spawn(async move {
        let mut hasher = Sha256::new();
        let mut off = 0;

        loop {
            let buf = match hr_rx.recv().await {
                Some(buf) => buf,
                None => break
            };

            hasher.update(&buf[..]);
            off += buf.len() as u64;
            info!("{off} {}", buf.len());

            let _ = rh_tx.send(buf).await;
        }

        let sha256 = format!("{}", hex::encode(hasher.finalize()));
        Ok((sha256, off))
    });

    let wfut: tokio::task::JoinHandle<Result<_, io::Error>> = tokio::spawn(async move {
        let mut buf = Arc::new(Vec::with_capacity(32768));

        loop {
            // we don't care if this is received; it won't be in the
            // case where the input stream is empty
            let _ = rw_tx.send(buf).await;
            buf = match wr_rx.recv().await {
                Some(buf) => buf,
                None => break
            };

            writer.write_all(&buf[..]).await?;
        }

        Ok(())
    });

    rfut.await??;
//    wfut.await?
    wfut.await??;
    let (sha256, off) = hfut.await??;
    Ok((sha256, off))
}

/*
use std::io::Write;

pub async fn copy_stream_to_writer<S, W>(
    stream: S,
    mut writer: W
) -> Result<(String, u64), io::Error>
where
    S: Stream<Item = Result<Bytes, io::Error>> + Send + Unpin + 'static,
    W: Write + Send + 'static
{
    use tokio::runtime::Handle;

    let handle = Handle::current();

    let (sha256, off) = tokio::task::spawn_blocking(move || {
        let mut reader = StreamReader::new(stream);

        let mut hasher = Sha256::new();
        let mut buf = vec![0; 8192];
        let mut off = 0u64;

        loop {
            match handle.block_on((&mut reader).read(&mut buf)) {
                Ok(0) => {
                    let sha256 = format!("{:x}", hasher.finalize());
                    break Ok((sha256, off))
                },
                Ok(r) => {
                    hasher.update(&buf[..r]);
                    writer.write(&buf[..r])?;
                    off += r as u64;
                    info!("{off} {r}");
                },
                Err(e) => break Err(e)
            }
        }
    }).await??;

    Ok((sha256, off))
}
*/

/*
async fn copy_stream_to_writer<S, W>(
    mut stream: S,
    mut writer: W
) ->  Result<u64, io::Error>
where
    S: Stream<Item = Result<Bytes, io::Error>> + Send + Unpin + 'static,
    W: AsyncWrite + Send + Unpin + 'static
{
    let (w_tx, mut w_rx) = tokio::sync::mpsc::channel(1);

    let rfut: tokio::task::JoinHandle<Result<_, io::Error>> = tokio::spawn(async move {
        use futures::StreamExt;
        while let Some(r) = stream.next().await {
            match r {
                Err(e) => return Err(e),
                Ok(buf) => { w_tx.send(buf).await.map_err(io::Error::other)?; }
            }
        }

        Ok(())
    });

    let wfut: tokio::task::JoinHandle<Result<_, io::Error>> = tokio::spawn(async move {
        let mut off = 0;

        loop {
            let buf = match w_rx.recv().await {
                Some(buf) => buf,
                None => break
            };

            writer.write_all(&buf[..]).await?;
            off += buf.len() as u64;
            info!("{off} {}", buf.len());
        }

        Ok(off)
    });

    rfut.await??;
    let off = wfut.await??;

    Ok(off)
}
*/

async fn hash_reader<R>(
    mut reader: R
) -> Result<(String, u64), io::Error>
where
    R: AsyncRead + Send + Unpin + 'static
{
    use tokio::runtime::Handle;
    let handle = Handle::current();

    let (sha256, off) = tokio::task::spawn_blocking(move || {
        let mut hasher = Sha256::new();
        let mut buf = vec![0; 8192];
        let mut off = 0u64;

        loop {
            match handle.block_on((&mut reader).read(&mut buf)) {
                Ok(0) => {
                    let sha256 = format!("{}", hex::encode(hasher.finalize()));
                    break Ok((sha256, off))
                },
                Ok(r) => {
                    hasher.update(&buf[..r]);
                    off += r as u64;
                },
                Err(e) => break Err(e)
            }
        }
    }).await??;

    Ok((sha256, off))
}

async fn stream_to_temp_file<S>(
    filename: &str,
    upload_dir: &std::path::Path,
    stream: S
) -> Result<(TempFile, u64, String), AppError>
where
    S: Stream<Item = Result<Bytes, io::Error>> + Send + Unpin + 'static
{
    // ensure the filename is valid
    let filename = safe_filename(filename)
        .or(Err(AppError::MalformedQuery))?;

    let mut file = TempFile::new_in(upload_dir)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let file_path = file.file_path().to_owned();

    info!("created temp file {}", file_path.display());

    let writer = file.try_clone()
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let start_time = std::time::Instant::now();

/*
    let writer = std::fs::File::create(&file_path)
        .map_err(|e| AppError::InternalError(e.to_string()))?;
*/

    let (sha256, size) = copy_stream_to_writer(stream, writer)
        .await
        .map_err(|e| match e.kind() {
            io::ErrorKind::FileTooLarge => AppError::TooLarge,
            _ => AppError::InternalError(e.to_string())
        })?;

/*
    let size = copy_stream_to_writer(stream, writer)
        .await
        .map_err(|e| match e.kind() {
            io::ErrorKind::FileTooLarge => AppError::TooLarge,
            _ => AppError::InternalError(e.to_string())
        })?;
*/

    let elapsed = start_time.elapsed().as_secs_f32();
    info!("{size}/{} = {}", elapsed, size as f32 / elapsed / 1024.0 / 1024.0 * 8.0);

/*
    file.rewind()
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let reader = file.try_clone()
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let (sha256, size) = hash_reader(reader)
        .await
        .map_err(|e| AppError::InternalError(e.to_string()))?;
*/

    info!("wrote temp file {}", file_path.display());

    Ok((file, size, sha256))
}

pub async fn file_post(
    Owned(owner, proj): Owned,
    ProjectPackageRelease(_, _, release): ProjectPackageRelease,
    Path((_, _, _, filename)): Path<(String, String, String, String)>,
    content_type: Option<TypedHeader<ContentType>>,
    content_length: Option<TypedHeader<ContentLength>>,
    State(core): State<CoreArc>,
    request: Request
) -> Result<(), AppError>
{
    let limit = limit_content_length(
        content_length.map(|cl| cl.0.0),
        core.max_file_size()
    )?;

    let (mut file, size, sha256) = stream_to_temp_file(
        &filename,
        core.upload_dir(),
        into_limited_stream(request, limit)
    ).await?;

    let path = file.file_path().to_owned();

    Ok(
        core.add_file(
             owner,
             proj,
             release,
             &filename,
             content_type.map(|h| h.0.into()).as_ref(),
             size,
             &sha256,
             &path,
             &mut file
        ).await?
    )
}

pub async fn gallery_post(
    Owned(owner, proj): Owned,
    Path((_, img_name)): Path<(String, String)>,
    content_type: Option<TypedHeader<ContentType>>,
    content_length: Option<TypedHeader<ContentLength>>,
    State(core): State<CoreArc>,
    request: Request
) -> Result<(), AppError>
{
    let limit = limit_content_length(
        content_length.map(|cl| cl.0.0),
        core.max_file_size()
    )?;

    // NB: No ContentType header will result in BAD_REQUEST by default, so
    // have to make it optional and check manually
    let content_type = content_type.ok_or(AppError::BadMimeType)?.0.into();

    let (mut file, size, sha256) = stream_to_temp_file(
        &img_name,
        core.upload_dir(),
        into_limited_stream(request, limit)
    ).await?;

    let path = file.file_path().to_owned();

    Ok(
        core.add_gallery_image(
            owner,
            proj,
            &img_name,
            &content_type,
            size,
            &sha256,
            &path,
            &mut file
        ).await?
    )
}

pub async fn gallery_patch(
    Owned(owner, proj): Owned,
    State(core): State<CoreArc>,
    Wrapper(Json(gallery_patch)): Wrapper<Json<GalleryPatch>>
) -> Result<(), AppError>
{
    Ok(core.update_gallery(owner, proj, &gallery_patch).await?)
}

pub async fn image_get(
    proj: Project,
    Path((_, img_name)): Path<(String, String)>,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
    Ok(Redirect::to(&core.get_image(proj, &img_name).await?))
}

pub async fn image_revision_get(
    proj: Project,
    Path((_, img_name, revision)): Path<(String, String, u32)>,
    State(core): State<CoreArc>
) -> Result<Redirect, AppError>
{
    Ok(
        Redirect::to(
            &core.get_image_revision(proj, revision as i64, &img_name).await?
        )
    )
}

pub async fn image_post(
    Owned(owner, proj): Owned,
    Path((_, img_name)): Path<(String, String)>,
    content_type: Option<TypedHeader<ContentType>>,
    content_length: Option<TypedHeader<ContentLength>>,
    State(core): State<CoreArc>,
    request: Request
) -> Result<(), AppError>
{
    let limit = limit_content_length(
        content_length.map(|cl| cl.0.0),
        core.max_file_size()
    )?;

    // NB: No ContentType header will result in BAD_REQUEST by default, so
    // have to make it optional and check manually
    let content_type = content_type.ok_or(AppError::BadMimeType)?.0.into();

    let (mut file, size, sha256) = stream_to_temp_file(
        &img_name,
        core.upload_dir(),
        into_limited_stream(request, limit)
    ).await?;

    let path = file.file_path().to_owned();

    Ok(
        core.add_image(
            owner,
            proj,
            &img_name,
            &content_type,
            size,
            &sha256,
            &path,
            &mut file
        ).await?
    )
}

pub async fn publishers_get(
    State(core): State<CoreArc>
) -> Result<Json<Publishers>, AppError>
{
    Ok(Json(core.get_publishers().await?))
}

pub async fn tags_get(
    State(core): State<CoreArc>
) -> Result<Json<Tags>, AppError>
{
    Ok(Json(core.get_tags().await?))
}

pub async fn flag_post(
    requester: User,
    proj: Project,
    State(core): State<CoreArc>,
    Wrapper(Json(flag)): Wrapper<Json<FlagPost>>,
) -> Result<(), AppError>
{
    Ok(core.add_flag(requester, proj, &flag).await?)
}

pub async fn admin_flag_close(
    admin: Admin,
    flag: Flag,
    State(core): State<CoreArc>
) -> Result<(), AppError>
{
    Ok(core.close_flag(admin, flag).await?)
}

pub async fn admin_flags_get(
    _admin: Admin,
    State(core): State<CoreArc>
) -> Result<Json<Flags>, AppError>
{
    Ok(Json(core.get_flags().await?))
}

pub async fn admin_user_event_post(
    State(core): State<CoreArc>,
    DiscourseEvent(data): DiscourseEvent<UserUpdatePost>
) -> Result<(), AppError>
{
    Ok(core.update_user(&data.user).await?)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn limit_content_length_under_limit() {
        let len = 20;
        assert_eq!(
            limit_content_length(Some(len as u64), len + 1).unwrap(),
            len
        );
    }

    #[test]
    fn limit_content_length_at_limit() {
        let len = 20;
        assert_eq!(
            limit_content_length(Some(len as u64), len).unwrap(),
            len
        );
    }

    #[test]
    fn limit_content_length_too_long() {
        let len = 20;
        assert_eq!(
            limit_content_length(Some((len as u64) + 1), len).unwrap_err(),
            AppError::TooLarge
        );
    }

    #[test]
    fn limit_content_length_way_too_long() {
        let len = 20;
        assert_eq!(
            limit_content_length(Some(u64::MAX), len).unwrap_err(),
            AppError::TooLarge
        );
    }

    #[test]
    fn limit_content_length_no_content_length() {
        let len = 20;
        assert_eq!(
            limit_content_length(None, len).unwrap(),
            len
        );
    }
}
