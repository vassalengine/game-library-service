use mime::Mime;
use std::path::Path;

pub fn mime_type_ok(mime: &Mime) -> bool {
    mime == &mime::IMAGE_PNG ||
    mime == &mime::IMAGE_GIF ||
    mime == &mime::IMAGE_JPEG ||
    mime == &mime::IMAGE_SVG ||
    (
        mime.type_() == mime::IMAGE && (
            mime.subtype() == "avif" ||
            mime.subtype() == "webp"
        )
    )
}

fn extension_ok<P: AsRef<Path>>(path: P, info: &infer::Type) -> bool {
    path.as_ref().extension().is_some_and(|ext| ext == info.extension())
}

pub fn check_magic<P: AsRef<Path>>(
    path: P,
    buf: &[u8]
) -> bool
{
    // check that MIME type matches file extension and is an expected type
    match infer::get(buf) {
        Some(info) =>
            extension_ok(path, &info) && 
            info.mime_type().parse::<Mime>()
                .is_ok_and(|mime| mime_type_ok(&mime)),
        None => false
    }
}


mod test {
    use super::*;

    use std::fs;

    #[track_caller]
    fn do_check_magic(path: &str, result: bool) {
        assert_eq!(check_magic(path, &fs::read(path).unwrap()), result);
    }

    #[test]
    fn check_magic_png() {
        do_check_magic("test/a.png", true); 
    }

    #[test]
    fn check_magic_jpg() {
        do_check_magic("test/a.jpg", true); 
    }
    
    #[test]
    fn check_magic_gif() {
        do_check_magic("test/a.gif", true); 
    }
    
    #[test]
    fn check_magic_webp() {
        do_check_magic("test/a.webp", true); 
    }
    
    #[test]
    fn check_magic_avif() {
        do_check_magic("test/a.avif", true); 
    }
    
    #[test]
    fn check_magic_extension_type_mismatch() {
        do_check_magic("test/not.gif", false); 
    }

    #[test]
    fn check_magic_unrecognized() {
        do_check_magic("test/empty", false); 
    }
}
