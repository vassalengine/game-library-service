use std::{
    io::{self, Read},
    fs::File,
    path::{Path, PathBuf}
};
use zip::{
    ZipArchive,
    result::ZipError
};

use crate::version::{MalformedVersion, Version};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Zip(#[from] ZipError),
    #[error("{0}")]
    Xml(#[from] sxd_document::parser::Error),
    #[error("{0}")]
    Xpath(#[from] sxd_xpath::Error),
    #[error("{0}")]
    Version(#[from] MalformedVersion)
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        // io::Error, ZipError, semver::Error are not PartialEq
        match (self, other) {
            (Self::Xml(l), Self::Xml(r)) => l == r,
            (Self::Xpath(l), Self::Xpath(r)) => l == r,
            _ => false
        }
    }
}

fn dump_file<P: AsRef<Path>>(
    zippath: P,
    filepath: &str
) -> Result<String, Error>
{
    // open module as zip archive
    let zipfile = File::open(zippath)?;
    let mut archive = ZipArchive::new(zipfile)?;

    // read file
    let mut file = archive.by_name(filepath)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    Ok(data)
}

fn version_in_moduledata(md: &str) -> Result<String, Error> {
    // extract <version> from moduledata
    let package = sxd_document::parser::parse(md)?;
    let document = package.as_document();
    let value = sxd_xpath::evaluate_xpath(&document, "/data/version")?;
    Ok(value.string())
}

fn check_version_impl<P: AsRef<Path>>(path: P) -> Result<Version, Error> {
    let md = dump_file(path, "moduledata")?;
    Ok(version_in_moduledata(&md)?.parse::<Version>()?)
}

pub async fn check_version<P: Into<PathBuf>>(
    path: P
) -> Result<Version, Error>
{
    let path = path.into();
    match tokio::task::spawn_blocking(move || check_version_impl(path)).await {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(Error::Io(io::Error::from(e)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dump_file_ok() {
        assert_ne!(
            dump_file("test/test.vmod", "moduledata").unwrap(),
            ""
        );
    }

    #[test]
    fn dump_file_zip_not_found() {
        assert!(
            matches!(
                dump_file("test/bogus.zip", "whatever").unwrap_err(),
                Error::Io(_)
            )
        );
    }

    #[test]
    fn dump_file_not_a_zip() {
        assert!(
            matches!(
                dump_file("test/empty", "whatever").unwrap_err(),
                Error::Zip(_)
            )
        );
    }

    #[test]
    fn version_in_moduledata_ok() {
        let md = "<data><version>0.0</version></data>";
        assert_eq!(
            version_in_moduledata(md).unwrap(),
            "0.0"
        );
    }

    #[test]
    fn version_in_moduledata_bad_xml() {
        let md = "<data>";
        assert!(
            matches!(
                version_in_moduledata(md).unwrap_err(),
                Error::Xml(_)
            )
        );
    }

    #[test]
    fn version_in_moduledata_missing_version() {
        let md = "<data></data>";
        assert_eq!(
            version_in_moduledata(md).unwrap(),
            ""
        );
    }
}
