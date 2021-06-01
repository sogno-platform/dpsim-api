use rocket::local::blocking::Client;
use rocket::Build;
use serde::{Deserialize, Serialize};
use super::{post_simulation, get_simulation, incomplete_form};
use rocket::http::ContentType;
// need this to read the file into the
// byte array for the multipart test
use std::fs;
use bytes::BufMut;

#[launch]
fn rocket() -> rocket::Rocket<Build> {
    rocket::build()
        .register("/", catchers![incomplete_form])
        .mount("/", routes![get_simulation, post_simulation])
}

#[test]
fn test_get_simulation() {
    // Construct a client to use for dispatching requests.
    let client = Client::untracked(rocket()).expect("valid rocket instance");

    // Dispatch a request to 'GET /' and validate the response.
    let response = client.get("/simulation").dispatch();
    let reply = response.into_string().unwrap();
    assert_eq!(reply, "Could not write to redis DB: Connection refused (os error 111)");
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SimulationPost<> {
    simulation_type: String,
    load_profile_data: String,
    model_id: u64
}

fn make_post_with_zip(zip_filename: &str, boundary: &[u8]) -> Vec<u8> {

    let zipdata = fs::read(zip_filename).unwrap();
    let mut buf = vec![];
    buf.put(boundary);
    buf.put(&b"\r\n"[..]);
    buf.put(&b"Content-Disposition: form-data; name=\"model_id\"\r\n"[..]);
    buf.put(&b"\r\n"[..]);
    buf.put(&b"1\r\n"[..]);
    buf.put(boundary);
    buf.put(&b"\r\n"[..]);
    buf.put(&b"Content-Disposition: form-data; name=\"simulation_type\"\r\n"[..]);
    buf.put(&b"\r\n"[..]);
    buf.put(&b"Powerflow\r\n"[..]);
    buf.put(boundary);
    buf.put(&b"\r\n"[..]);
    buf.put(&b"Content-Disposition: form-data; name=\"load_profile_data\"; filename=\"load_pd.zip\"\r\n"[..]);
    buf.put(&b"Content-Type: application/json\r\n"[..]);
    buf.put(&b"\r\n"[..]);
    buf.put(&*zipdata);
    buf.put(&b"\r\n"[..]);
    buf.put(boundary);
    buf.put(&b"--\r\n"[..]);
    buf.put(&b"\r\n"[..]);
    buf
}

// multipart post code based on the example at https://github.com/SergioBenitez/Rocket/issues/1591
#[test]
fn test_post_simulation() {

    let client = Client::untracked(rocket()).expect("valid rocket instance");

    let ct = "multipart/form-data; boundary=X-BOUNDARY"
        .parse::<ContentType>()
        .unwrap();
   
    let body = make_post_with_zip("./testdata/load_profile_data.zip", "--X-BOUNDARY".as_bytes());

    let response = client.post("/simulation")
        .header(ct)
        .remote("127.0.0.1:8000".parse().unwrap())
        .body(&body)
        .dispatch();

    let reply = response.into_string().unwrap();
    let expected_json = "{\"simulation_id\":0,\"simulation_type\":\"Powerflow\",\"model_id\":1}";
    assert_eq!(expected_json, reply)
}