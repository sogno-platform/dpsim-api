use rocket::local::blocking::Client;
use rocket::Build;
use serde::{Deserialize, Serialize};
use crate::routes::{Simulation, SimulationType, get_routes, incomplete_form};
use rocket::http::ContentType;
use serde_json::json;
use assert_json_diff::assert_json_eq;
use crate::Template;
use crate::routes::{SimulationForm};

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
        error:           "".to_string(),
        load_profile_id: "".to_string(),
        model_id:        "1".to_string(),
        results_id:      "1".to_string(),
        results_data:    "".to_string(),
        simulation_id:   1,
        simulation_type: SimulationType::Powerflow,
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
        error:           "".to_string(),
        load_profile_id: "".to_string(),
        model_id:        "1".to_string(),
        results_id:      "1".to_string(),
        results_data:    r#"{
  "data": {
    "fileID": "d297cb7c-b578-4da8-9d79-76432e8986e9",
    "lastModified": "2022-04-27T09:27:09Z",
    "url": "http://minio:9000/sogno-platform/d297cb7c-b578-4da8-9d79-76432e8986e9?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=ALFRED123%2F20220427%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20220427T092744Z&X-Amz-Expires=604800&X-Amz-SignedHeaders=host&X-Amz-Signature=48ec50cecae04ca34693429bc740f9a0b81e829b6e696800e823399355ab83b9"
  }
}
"#.to_string(),
        simulation_id:   1,
        simulation_type: SimulationType::Powerflow,
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
    load_profile_id: String,
    model_id:        u64
}

#[test]
fn test_post_simulation() {
    let client = Client::untracked(rocket()).expect("valid rocket instance");

    let ct = "application/json"
        .parse::<ContentType>()
        .unwrap();
   
    let form = SimulationForm { model_id: "1".to_string(), load_profile_id: "1".to_string(), simulation_type: SimulationType::Powerflow.into() };
    let body = serde_json::to_string(&form).unwrap();
    let response = client.post("/simulation")
        .header(ct)
        .remote("127.0.0.1:8000".parse().unwrap())
        .body(&body)
        .dispatch();

    let reply = response.into_string().unwrap();
    println!("REPLY: {:?}", reply);
    let expected_simulation = json!(Simulation {
        error:             "".to_string(),
        load_profile_id:   "1".to_string(),
        model_id:          "1".to_string(),
        results_id:        "100".to_string(),
        results_data:      "".to_string(),
        simulation_id:     1,
        simulation_type:   SimulationType::Powerflow,
    });
    let received_json: Simulation = serde_json::from_str( reply.as_str() ).unwrap();
    assert_json_eq!(expected_simulation, received_json)
}
