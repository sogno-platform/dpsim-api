
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::http::Status;
use async_global_executor::block_on;
use crate::db;
use crate::amqp;
use serde_json::json;
use rocket_dyn_templates::{Template};
use paste::paste;
use crate::{Simulation, SimulationForm, SimulationType};
use serde::{Serialize};

macro_rules! create_endpoint_with_doc{
    (
        #[describe($description:tt)]
        #[example($wget:tt)]
        #[post($route_path:tt, format = $content_type:tt, data = "<form>")]
        $(#[$attr:meta])*
        pub async fn $name:ident( $($arg_name:ident : $arg_ty:ty),* $(,)? ) $(-> $ret:ty)?
            $body:block
    ) => {
        paste! {
            #[get($route_path, rank=2, format = "text/html")]
            pub async fn [<get_ $name>] (  ) -> Template {
                document_link_page(stringify!( $name ))
            }
        }
        paste! {
            #[ doc = $description]
            #[ doc = " * Content-Type: " $content_type]
            #[ doc = "### Example request:" ]
            #[ doc = "```bash" ]
            #[ doc = $wget ]
            #[ doc = "```" ]
            #[post($route_path, format = $content_type, data = "<form>")]
            $( #[$attr] )*
            pub async fn $name ( $($arg_name : $arg_ty),* ) $(-> $ret)?
                $body
        }
    };
    (
        #[describe($description:tt)]
        #[example($wget:tt)]
        #[get($route_path:tt, format = $content_type:tt)]
        $(#[$attr:meta])*
        pub async fn $name:ident( $($arg_name:ident : $arg_ty:ty),* $(,)? ) $(-> $ret:ty)?
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
            #[get($route_path, format = $content_type)]
            $( #[$attr] )*
            pub async fn $name ( $($arg_name : $arg_ty),* ) $(-> $ret)?
                $body
        }
    }
}

#[doc = "A struct used for sharing info about a route with a template"]
#[derive(Serialize, Clone)]
struct Route {
    collapse_id: String,
    heading_id: String,
    link: String,
    method: String,
    name: String,
    path: String
}

#[derive(Serialize)]
#[doc = "A struct used for sharing info about some routes with a template"]
struct RoutesContext {
    routes: Vec<Route>
}

create_endpoint_with_doc!(
    #[describe("List the endpoints")]
    #[example("curl -X GET -H 'Accept: application/json' http://localhost:8000/api")]
    #[get("/api", format = "text/html")]
    pub async fn get_api() -> Template {
        let mut routes = [].to_vec();
        for (index, fn_name) in get_routes().iter().enumerate() {
            let heading_id  = format!("heading{}", index);
            let collapse_id  = format!("collapse{}", index);
            let name = match &fn_name.name {
                Some(x) => x.to_string(),
                None => "".to_string()
            };
            let route_json = Route {
                collapse_id: collapse_id,
                heading_id: heading_id,
                link: document_link(&name),
                method: fn_name.method.to_string(),
                name: name,
                path: fn_name.uri.path().to_string(),
            };
            routes.push(route_json);
        }
        let context = RoutesContext{ routes: routes };
        Template::render("api", &context)
    }
);

create_endpoint_with_doc!(
    #[describe("Redirects to /api")]
    #[example("curl -X GET -H 'Accept: application/json' http://localhost:8000/")]
    #[get("/", format = "text/html")]
    pub async fn get_root() -> Redirect {
        Redirect::to(uri!(get_api))
    }
);

create_endpoint_with_doc!(
    #[describe("Show details for a simulation")]
    #[example("curl -X GET -H 'Accept: application/json' http://localhost:8000/simulation/1")]
    #[get("/simulation/<id>", format="application/json")]
    pub async fn get_simulation_id(id: u64) -> (Status, Result<Json<Simulation>, String>) {
        match db::read_simulation(id) {
            Ok(sim) => (Status::Ok, Ok(Json(sim))),
            Err(e) =>  (Status::NotFound, Err(e.to_string()))
        }
    }
);

create_endpoint_with_doc!(
    #[describe("List the simulations")]
    #[example("curl -X GET -H 'Accept: application/json' http://localhost:8000/simulation")]
    #[get("/simulation", format="application/json")]
    pub async fn get_simulations() -> (Status, Result<Json<Simulation>, String>) {
        match db::write_u64(&String::from("redis_key"), 36u64) {
            Ok(_) => (),
            Err(e) => return (Status::Unauthorized, Err(format!("Could not write to redis DB: {}", e)))
        };
        match db::read_u64(&String::from("redis_key")) {
            Ok(i) => (Status::Ok, Ok(Json(Simulation {
                error: "".to_string(),
                load_profile_data: [].to_vec(),
                model_id: i,
                simulation_id: 1,
                simulation_type: SimulationType::Powerflow
            }))),
            Err(e) => (Status::UnprocessableEntity, Err(format!("Could not read from redis DB: {}", e)))
        }
    }
);

create_endpoint_with_doc!(
    #[describe("Create a new simulation")]
    #[example("
        curl -X POST
             -H 'Accept: application/json'
             -F file=@testdata/load_profile_data.zip
             -F model_id=1
             -F allowed_file_types=application/zip hocalhost:8000/simulation"
    )]
    #[post("/simulation", format = "multipart/form-data", data = "<form>")]
    pub async fn post_simulation(form: Form < SimulationForm < '_ > > ) -> (Status, Result<Json<Simulation>, String>) {
        match super::parse_simulation_form(form).await {
            Ok(simulation) => {
                match block_on(amqp::publish(&simulation)) {
                    Ok(()) =>(Status::Accepted, Ok(simulation)),
                    Err(e) =>(Status::BadGateway, Err(format!("Could not publish to amqp server: {} ", e.to_string())))
                }
            },
            Err(e) => (Status::NotAcceptable, Err(format!("Error when parsing form: {} ", e.to_string())))
        }
    }
);

#[doc = "Create a link to the documentation page for the given function"]
fn document_link(fn_name: &str) -> String {
    format!("https://sogno-platform.github.io/dpsim-api/dpsim_api/routes/fn.{}{}", fn_name, ".html")
}

#[doc = "Create an html button linking to the documentation page for the given function"]
fn document_link_page(fn_name: &str) -> Template {
    let uri = document_link(fn_name);
    let context = json!({ "name": fn_name.to_string(), "link": uri });
    Template::render("doc_link", &context)
}

#[doc = "Handler for when an incomplete form has been submitted"]
#[catch(422)]
pub async fn incomplete_form(form: &rocket::Request<'_>) -> String {
    info!("FORM: {}", form);
    "Incomplete form".to_string()
}

#[doc = "Returns the list of routes that we have defined"]
pub fn get_routes() -> Vec<rocket::Route>{
    return routes![get_root, get_api, get_simulations, post_simulation, get_post_simulation, get_simulation_id]
}
