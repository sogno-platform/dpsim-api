
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::http::Status;
use async_global_executor::block_on;
use crate::db;
use crate::amqp;

macro_rules! create_api_with_doc{
    (
        #[describe($description:tt)]
        #[example($wget:tt)]
        #[post($route_path:tt, format = $content_type:tt, data = "<form>")]
        $(#[$attr:meta])*
        pub async fn $name:ident( $(mut $arg_name:ident : $arg_ty:ty),* $(,)? ) $(-> $ret:ty)?
            $body:block
    ) => {
        doc_comment! {
            concat!(
                $description, "\n",
                " * Content-Type: ", $content_type, "\n",
            ),
            #[ doc = "### Example request:" ]
            #[ doc = "```bash" ]
            #[ doc = $wget ]
            #[ doc = "```" ]
            #[post($route_path, format = $content_type, data = "<form>")]
            $( #[$attr] )*
            pub async fn $name ( $(mut $arg_name : $arg_ty),* ) $(-> $ret)?
                $body
        }
    };
    (
        #[describe($description:tt)]
        #[example($wget:tt)]
        #[get($route_path:tt)]
        $(#[$attr:meta])*
        pub async fn $name:ident( $(mut $arg_name:ident : $arg_ty:ty),* $(,)? ) $(-> $ret:ty)?
            $body:block
    ) => {
        doc_comment! {
            concat!(
                $description, "\n"
            ),
            #[ doc = "### Example request:" ]
            #[ doc = "```bash" ]
            #[ doc = $wget ]
            #[ doc = "```" ]
            #[get($route_path)]
            $( #[$attr] )*
            pub async fn $name ( $(mut $arg_name : $arg_ty),* ) $(-> $ret)?
                $body
        }
    }
}

fn error_simulation(message: String) -> super::Simulation {
    return super::Simulation {
        error: message,
        model_id: 0,
        simulation_id: 0,
        simulation_type: "".to_string(),
        load_profile_data: [].to_vec()
    }
}

create_api_with_doc!(
    #[describe("List the endpoints")]
    #[example("curl -X GET -H 'Accept: application/json' http://localhost:8000/api")]
    #[get("/api")]
    pub async fn get_api() -> String {
        let mut names: String = "".to_string();
        for fn_name in get_routes() {
            names.push_str(&fn_name.method.to_string());
            names.push_str(" ");
            names.push_str(&fn_name.uri.path());
            names.push_str("\n");
        }
        names
    }
);

create_api_with_doc!(
    #[describe("Redirects to /api")]
    #[example("curl -X GET -H 'Accept: application/json' http://localhost:8000/")]
    #[get("/")]
    pub async fn get_root() -> Redirect {
        Redirect::to(uri!(get_api))
    }
);

create_api_with_doc!(
    #[describe("List the simulations")]
    #[example("curl -X GET -H 'Accept: application/json' http://localhost:8000/simulation")]
    #[get("/simulation", format="application/json")]
    pub async fn get_simulation() -> (Status, Json<super::Simulation>) {
        match db::write_u64(&String::from("redis_key"), 36u64) {
            Ok(_) => (),
            Err(e) => return (Status::Unauthorized, Json(error_simulation(format!("Could not write to redis DB: {}", e))))
        };
        match db::read_u64(&String::from("redis_key")) {
            Ok(i) => (Status::Ok, Json(super::Simulation { error: "".to_string(), model_id: i, simulation_id: 0, simulation_type: "Powerflow".to_string(), load_profile_data: [].to_vec() })),
            Err(e) => return (Status::UnprocessableEntity, Json(error_simulation(format!("Could not read from redis DB: {}", e))))
        }
    }
);

create_api_with_doc!(
    #[describe("Create a new simulation")]
    #[example("
        curl -X POST
             -H 'Accept: application/json'
             -F file=@testdata/load_profile_data.zip
             -F model_id=1
             -F allowed_file_types=application/zip hocalhost:8000/simulation"
    )]
    #[post("/simulation", format = "multipart/form-data", data = "<form>")]
    pub async fn post_simulation(mut form: Form < super::SimulationForm < '_ > > ) -> (Status, Json<super::Simulation>) {
        match super::parse_simulation_form(form).await {
            Ok(simulation) => {
                match block_on(amqp::publish(&simulation)) {
                    Ok(()) =>(Status::Accepted, simulation),
                    Err(e) =>(Status::BadGateway, Json(error_simulation(format!("Could not publish to amqp server: {} ", e.to_string()))))
                }
            },
            Err(e) => (Status::NotAcceptable, Json(error_simulation(format!("Error when parsing form: {} ", e.to_string()))))
        }
    }
);

#[catch(422)]
pub async fn incomplete_form(form: &rocket::Request<'_>) -> String {
    info!("FORM: {}", form);
    "Incomplete form".to_string()
}

pub fn get_routes() -> Vec<rocket::Route>{
    return routes![get_root, get_api, get_simulation, post_simulation]
}
