
#[cfg(not(test))]
use hyper::{body::HttpBody as _};
use bytes::{BytesMut,Bytes};
#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::io::Read;

#[cfg(test)]
pub async fn get_data_from_url(url: &str) -> Result<Box<Bytes>, hyper::Error> {
    println!("get_data_from_url{:?}", url);
    let mut f = File::open("testdata/file_service_test.json").unwrap();
    let mut buf = BytesMut::with_capacity(1024*10);
    buf.resize(2014 * 10, 0);
    let count = f.read(&mut buf[..]).unwrap();
    println!("read {:?}", count);
    buf.truncate(count);
    let frozen = buf.freeze();
    Ok(Box::new(frozen))
}

#[cfg(not(test))]
pub async fn get_data_from_url(url: &str) -> Result<Box<Bytes>, hyper::Error> {
    let client = hyper::Client::new();
    let uri = url.parse::<hyper::Uri>().unwrap();

    // Await the response...
    let mut resp = client.get(uri).await?;
    println!("Response: {}", resp.status());

    let body = resp.body_mut();
    let mut buf = BytesMut::with_capacity(body.size_hint().lower() as usize);
    while let Some(chunk) = body.data().await {
        buf.extend_from_slice(&chunk?);
    }
    let frozen = buf.freeze();
    Ok(Box::new(frozen))
}

#[doc = "Function to get a URL from sogno-file-service using a file ID"]
pub async fn convert_id_to_url(model_id: &str) -> Result<String, hyper::Error>{

    // Parse an `http::Uri`...
    let model_id_url = format!("http://sogno-file-service:8080/api/files/{}", model_id);
    let data = get_data_from_url(&model_id_url).await;
    let url = match data {
        Ok(boxed_data) => {
            let body = std::str::from_utf8(&boxed_data).unwrap();
            let body_json: serde_json::Value = serde_json::from_str(body).unwrap();
            let url_str = body_json["data"]["url"].as_str().unwrap();
            url_str.into()
        },
        Err(error) => {
            println!("ERROR: {}", error);
            error.to_string()
        }
    };

    Ok(url)
}


