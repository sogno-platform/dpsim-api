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
        simulation_id:   1,
        simulation_type: SimulationType::Powerflow,
        model_id:        "1".to_string(),
        load_profile_id: "".to_string()
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
        simulation_id:   1,
        simulation_type: SimulationType::Powerflow,
        model_id:        "1".to_string(),
        load_profile_id: "".to_string()
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

// multipart post code based on the example at https://github.com/SergioBenitez/Rocket/issues/1591
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
        simulation_id:     1,
        simulation_type:   SimulationType::Powerflow,
        model_id:          "1".to_string(),
        load_profile_id:   "1".to_string()
    });
    let received_json: Simulation = serde_json::from_str( reply.as_str() ).unwrap();
    assert_json_eq!(expected_simulation, received_json)
}
