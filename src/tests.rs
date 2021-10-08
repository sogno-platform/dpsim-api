use rocket::local::blocking::Client;
use rocket::Build;
use serde::{Deserialize, Serialize};
use crate::routes::{get_routes, incomplete_form};
use crate::{Simulation, SimulationType};
use rocket::http::ContentType;
// need this to read the file into the
// byte array for the multipart test
use std::fs;
use bytes::BufMut;
use serde_json::json;
use assert_json_diff::assert_json_eq;
use crate::Template;

#[launch]
fn rocket() -> rocket::Rocket<Build> {
    rocket::build()
        .register("/", catchers![incomplete_form])
        .mount("/", get_routes())
        .attach(Template::fairing())
}

#[test]
fn test_get_simulation() {
    // Construct a client to use for dispatching requests.
    let client = Client::untracked(rocket()).expect("valid rocket instance");

    // Dispatch a request to 'GET /' and validate the response.
    let response = client.get("/simulation").dispatch();
    assert_eq!(response.status().code, 200);
    let reply = response.into_string().unwrap();
    let received_json: Simulation = serde_json::from_str( reply.as_str() ).unwrap();
    let expected_json = Simulation {
        error: "".to_string(),
        simulation_id: 1,
        simulation_type: SimulationType::Powerflow,
        model_id: 36,
        load_profile_data: [].to_vec()
    };
    assert_json_eq!(received_json, expected_json)
}

#[test]
fn test_get_simulation_by_id() {
    // Construct a client to use for dispatching requests.
    let client = Client::untracked(rocket()).expect("valid rocket instance");

    // Dispatch a request to 'GET /' and validate the response.
    let response = client.get("/simulation/1").dispatch();
    assert_eq!(response.status().code, 200);
    let reply = response.into_string().unwrap();
    let received_json: Simulation = serde_json::from_str( reply.as_str() ).unwrap();
    let expected_json = Simulation {
        error: "".to_string(),
        simulation_id: 1,
        simulation_type: SimulationType::Powerflow,
        model_id: 1,
        load_profile_data: [].to_vec()
    };
    assert_json_eq!(received_json, expected_json)
}

#[test]
fn test_get_openapi() {
    // Construct a client to use for dispatching requests.
    let client = Client::untracked(rocket()).expect("valid rocket instance");

    // Dispatch a request to 'GET /' and validate the response.
    let response = client.get("/openapi.json").dispatch();
    assert_eq!(response.status().code, 200);
    let reply = response.into_string().unwrap();
    let received_json: serde_json::Value = serde_json::from_str( reply.as_str() ).unwrap();
    println!("OPENAPI: {}", received_json)
}


#[derive(Serialize, Deserialize, Debug)]
pub struct SimulationPost<> {
    simulation_type: String,
    load_profile_data: Vec<u8>,
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
    buf.put(&b"Content-Type: application/zip\r\n"[..]);
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
    let expected_simulation = json!(super::Simulation {
        error:             "".to_string(),
        simulation_id:     1,
        simulation_type:   SimulationType::Powerflow,
        model_id:          1,
        load_profile_data:
            vec![
                "Load_H_1.csv".to_string(),  "Load_H_10.csv".to_string(), "Load_H_11.csv".to_string(),
                "Load_H_12.csv".to_string(), "Load_H_14.csv".to_string(), "Load_H_3.csv".to_string(),
                "Load_H_4.csv".to_string(),  "Load_H_5.csv".to_string(),  "Load_H_6.csv".to_string(),
                "Load_H_8.csv".to_string(),  "Load_I_1.csv".to_string(),  "Load_I_10.csv".to_string(),
                "Load_I_12.csv".to_string(), "Load_I_13.csv".to_string(), "Load_I_14.csv".to_string(),
                "Load_I_3.csv".to_string(),  "Load_I_7.csv".to_string(), "Load_I_9.csv".to_string(),
                "README.MD".to_string()
            ]
    });
    let mut received_json: super::Simulation = serde_json::from_str( reply.as_str() ).unwrap();
    received_json.load_profile_data.sort();
    assert_json_eq!(expected_simulation, received_json)
}
