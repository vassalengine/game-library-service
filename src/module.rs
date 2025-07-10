use std::{
    io::{self, Read},
    fs::File,
    path::Path
};
use sxd_xpath::Value;
use zip::{
    ZipArchive,
    result::ZipError
};

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

pub async fn dump_moduledata<P: AsRef<Path>>(
    path: P
) -> Result<String, Error>
{
    let path = path.as_ref().to_path_buf();
    match tokio::task::spawn_blocking(move || dump_file(path, "moduledata")).await {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(Error::Io(io::Error::from(e)))
    }
}

pub fn versions_in_moduledata(
    md: &str
) -> Result<(Option<String>, Option<String>), Error>
{
    let package = sxd_document::parser::parse(md)?;
    let document = package.as_document();

    Ok((
        // extract <version> from moduledata
        sxd_xpath::evaluate_xpath(&document, "/data/version")
            .ok()
            .map(Value::into_string)
            .filter(|s| !s.is_empty()),
        // extract <VassalVersion> from moduledata
        sxd_xpath::evaluate_xpath(&document, "/data/VassalVersion")
            .ok()
            .map(Value::into_string)
            .filter(|s| !s.is_empty())
    ))
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
    fn versions_in_moduledata_module() {
        let md = "<data><version>0.0</version></data>";
        assert_eq!(
            versions_in_moduledata(md).unwrap(),
            (Some("0.0".into()), None)
        );
    }

    #[test]
    fn versions_in_moduledata_vassal() {
        let md = "<data><VassalVersion>0.0</VassalVersion></data>";
        assert_eq!(
            versions_in_moduledata(md).unwrap(),
            (None, Some("0.0".into()))
        );
    }

    #[test]
    fn versions_in_moduledata_both() {
        let md = "<data><version>0.1</version><VassalVersion>0.0</VassalVersion></data>";
        assert_eq!(
            versions_in_moduledata(md).unwrap(),
            (Some("0.1".into()), Some("0.0".into()))
        );
    }

    #[test]
    fn version_in_moduledata_bad_xml() {
        let md = "<data>";
        assert!(
            matches!(
                versions_in_moduledata(md).unwrap_err(),
                Error::Xml(_)
            )
        );
    }

    #[test]
    fn version_in_moduledata_missing_version() {
        let md = "<data></data>";
        assert_eq!(
            versions_in_moduledata(md).unwrap(),
            (None, None)
        );
    }
}
