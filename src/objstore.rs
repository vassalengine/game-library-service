use object_store::{
    ObjectStore,
    aws::AmazonS3Builder,
    path::Path
};

#[cfg(test)]
mod test {
    use super::*;

    use futures_util::StreamExt;

    #[tokio::test]
    async fn xxx() {
        
        let s3 = AmazonS3Builder::new()
            .with_region("us-east-1")
            .with_bucket_name("obj.vassalengine.org")
            .with_endpoint("https://us-east-1.linodeobjects.com")
            .with_access_key_id("XBD4GB7KKB20444FSX2G")
            .with_secret_access_key("Za8FThDq7tbLPZe9Ef5BSiYuqnpnJFBgJD4ODDi2")
            .build()
            .unwrap();

        let prefix = Path::from("tracker");
        let mut list_stream = s3.list(Some(&prefix));

        while let Some(meta) = list_stream.next().await.transpose().unwrap() {
            println!("Name: {}, size: {}", meta.location, meta.size);
        }
    }
}
