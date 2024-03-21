use serde::Deserialize;
use std::{
    io::{self, Read},
    fs::File
};
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
    Xpath(#[from] sxd_xpath::Error)
}

fn dump_file(zippath: &str, filepath: &str) -> Result<String, Error> {
    // open module as zip archive
    let zipfile = File::open(zippath)?;
    let mut archive = ZipArchive::new(zipfile)?;

    // read moduledata file
    let mut file = archive.by_name(filepath)?;
    let mut md = String::new();
    file.read_to_string(&mut md)?;
    Ok(md)
}

fn version_in_moduledata(md: &str) -> Result<String, Error> {
    // extract <version> from moduledata
    let package = sxd_document::parser::parse(&md)?;
    let document = package.as_document();
    let value = sxd_xpath::evaluate_xpath(&document, "/data/version")?;
    Ok(value.string())
}

pub fn extract_version(path: &str) -> Result<String, Error> {
    let md = dump_file(path, "moduledata")?;
    version_in_moduledata(&md)
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

    #[test]
    fn extract_version_ok() {
        assert_eq!(
            extract_version("test/test.vmod").unwrap(),
            "0.0"
        );
    }
}
