
extern crate hyper_multipart_rfc7578 as hyper_multipart;

#[cfg(not(test))]
use hyper::{body::HttpBody as _, Client, Request};
use bytes::{BytesMut,Bytes};
use hyper_multipart::client::{multipart};
use std::io::Cursor;
#[cfg(test)]
use std::{fs::File,io::Read};

#[cfg(not(test))]
pub async fn post_results_file() -> Result<Box<Bytes>, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let mut form = multipart::Form::default();
    let bytes = Cursor::new("{\"ready\":\"false\"}");
    form.add_reader_file("file", bytes, "ready.json");
    let req_builder = Request::post("http://sogno-file-service:8080/api/files");
    let req = form.set_body_convert::<hyper::Body, multipart::Body>(req_builder)
        .unwrap();
    let mut resp = client.request(req).await?;
    let body = resp.body_mut();
    let mut buf = BytesMut::with_capacity(body.size_hint().lower() as usize);
    while let Some(chunk) = body.data().await {
        buf.extend_from_slice(&chunk?);
    }
    let frozen = buf.freeze();
    Ok(Box::new(frozen))
}

#[cfg(not(test))]
pub async fn create_results_file() -> Result<String, hyper::Error>{
    let data = post_results_file().await;
    let file_id = match data {
        Ok(boxed_data) => {
            let body = std::str::from_utf8(&boxed_data).unwrap();
            let body_json: serde_json::Value = serde_json::from_str(body).unwrap();
            if body_json.get("data".to_string()) != None {
                let url_str = body_json["data"]["fileID"].as_str().unwrap();
                url_str.into()
            }
            else {
                let error_str = body_json["error"]["message"].as_str().unwrap();
                error_str.into()
            }
        },
        Err(error) => {
            error.to_string()
        }
    };
    Ok(file_id)
}

#[cfg(test)]
pub async fn create_results_file() -> Result<String, hyper::Error>{
    let file_id:String = "100".to_string();
    Ok(file_id)
}

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
    let client = Client::new();
    let uri = url.parse::<hyper::Uri>().unwrap();

    // Await the response...
    let mut resp = client.get(uri).await?;
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
    let model_id_url = format!("http://sogno-file-service:8080/api/files/{}", model_id);
    let data = get_data_from_url(&model_id_url).await;
    let url = match data {
        Ok(boxed_data) => {
            let body = std::str::from_utf8(&boxed_data).unwrap();
            let body_json: serde_json::Value = serde_json::from_str(body).unwrap();
            if body_json.get("data".to_string()) != None {
                let url_str = body_json["data"]["url"].as_str().unwrap();
                url_str.into()
            }
            else {
                let error_str = body_json["error"]["message"].as_str().unwrap();
                error_str.into()
            }
        },
        Err(error) => {
            error.to_string()
        }
    };

    Ok(url)
}


