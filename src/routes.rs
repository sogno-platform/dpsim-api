use rocket::response::Redirect;
use rocket::serde::json::{Json, json};
use rocket::http::Status;
use async_global_executor::block_on;
use crate::db;
use crate::amqp;
use rocket_dyn_templates::{Template};
use paste::paste;
use crate::{Simulation, SimulationType};
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::io::prelude::*;
use std::ffi::OsStr;
use std::path::Path;
use rocket_okapi::{ openapi, OpenApiError,
                    response::OpenApiResponderInner,
                    gen::OpenApiGenerator };
use okapi::openapi3::Responses;
use serde::Serialize;
use rocket::http::{ContentType};
use rocket::{Request};
use rocket::response::{self, Responder, Response};
use rocket::fs::TempFile;
use rocket::form::Form;
use schemars::{schema_for, JsonSchema, gen::SchemaGenerator, schema::Schema};

#[doc = "Function to read a zip file"]
fn read_zip(reader: impl Read + Seek) -> zip::result::ZipResult<Vec<String>> {
    let mut zip = zip::ZipArchive::new(reader)?;
    let mut files = Vec::new();

    for i in 0..zip.len() {
        let file = zip.by_index(i)?;
        if file.is_file() {
            let path = Path::new(file.name());
            let stringy = match path.file_name() {
                Some(file_name) => osstr_to_string(file_name),
                None => "".into()
            };
            files.push(stringy);
        }
    }

    Ok(files)
}

fn osstr_to_string(str: &OsStr) -> String {
    let the_str = str.to_str().unwrap();
    String::from(the_str)
}

/// # Form for submitting a new Simulation
///
/// ## Parameters:
/// * simulation_type
///   - String
///   - must be one of "Powerflow", "Outage"
/// * load_profile_data
///   - ZIP file
///   - load profile data in CSV format
/// * model_id
///   - Integer
///   - must be a valid id that exists in the associated CIM service
//#[derive(FromForm, Debug, Default, Serialize, Deserialize)]
#[derive(FromForm, Debug)]
pub struct SimulationFormStruct<'f> {
    simulation_type: SimulationType,
    load_profile_data: TempFile<'f>,
    model_id: u64
}

impl<'a> schemars::JsonSchema for SimulationFormStruct <'a> {
    fn schema_name() -> String {
        "SimulationForm".to_owned()
    }
    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        schema_for!(SimulationFormStructJsonSchema).schema.into()
    }
}

#[allow(dead_code)]
#[derive(JsonSchema)]
pub struct SimulationFormStructJsonSchema {
    load_profile_data: Vec<u8>,
    model_id: u64,
    simulation_type: SimulationType
}

async fn parse_simulation_form(mut form: Form<SimulationFormStruct<'_>>) -> Result<Json<Simulation>, String>{
    let tmp_dir = PathBuf::from("/tmp/");
    let simulation_id = match db::get_new_simulation_id() {
        Ok(id) => id,
        Err(e) => return Err(format!("Failed to obtain new simulation id: {}", e))
    };
    let save_name = tmp_dir.join(format!("simulation_{}", simulation_id));
    match form.load_profile_data.persist_to(&save_name).await {
        Ok (()) => {
            let f = File::open(save_name).unwrap();
            match read_zip(f) {
                Ok(data) => {
                    let simulation = Simulation {
                        error: "".to_string(),
                        simulation_id: simulation_id,
                        simulation_type: form.simulation_type,
                        model_id: form.model_id,
                        load_profile_data: data
                    };
                    match db::write_simulation(&simulation_id.to_string(), &simulation) {
                        Ok(()) => Ok(Json(simulation)),
                        Err(e) => Err(format!("Could not write to db: {}", e.to_string()))
                    }
                }
                Err(e) => Err(format!("Failed to read zip: {}", e.to_string()))
            }
        },
        Err(e) => Err(format!("Failed to store load profile data: {}", e))
    }
}

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
            #[openapi(skip)]
            #[get($route_path, rank=2, format = "text/html")]
            pub async fn [<get_ $name>] (  ) -> Template {
                document_link_page(stringify!( $name ))
            }
        }
        paste! {
            #[allow(rustdoc::invalid_rust_codeblocks)]
            #[ doc = $description ]
            #[ doc = "\n\r * Content-Type: " $content_type ]
            #[ doc = "\n\r * Example request:\n\r" ]
            #[ doc = "```" ]
            #[ doc = $wget ]
            #[ doc = "```" ]
            #[openapi]
            #[post($route_path, format = $content_type, data = "<form>")]
            $( #[$attr] )*
            pub async fn $name ( $($arg_name : $arg_ty),* ) $(-> $ret)?
                $body
        }
    };
    (
        #[describe($description:tt)]
        #[example($wget:tt)]
        #[$openapi:meta]
        #[get($route_path:tt, format = $content_type:tt)]
        $(#[$attr:meta])*
        pub async fn $name:ident( $($arg_name:ident : $arg_ty:ty),* $(,)? ) $(-> $ret:ty)?
            $body:block
    ) => {
        #[allow(rustdoc::invalid_rust_codeblocks)]
        #[ doc = $description ]
        #[ doc = "\n\r * Example request:\n\r" ]
        #[ doc = "```" ]
        #[ doc = $wget ]
        #[ doc = "```" ]
        #[ $openapi ]
        #[get($route_path, format = $content_type)]
        $( #[$attr] )*
        pub async fn $name ( $($arg_name : $arg_ty),* ) $(-> $ret)?
            $body
    }
}

#[derive(Debug, Default, serde::Serialize, schemars::JsonSchema)]
pub struct SimulationError {
    pub err: String,
    #[serde(skip)]
    pub http_status_code: rocket::http::Status,
}

type SimulationResult = std::result::Result<Json<Simulation>, SimulationError>;

impl<'r> Responder<'r, 'static> for SimulationError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Convert object to json
        let body = serde_json::to_string(&self).unwrap();
        let len = body.len();
        Response::build()
            .sized_body(len, std::io::Cursor::new(body))
            .header(ContentType::JSON)
            .status(self.http_status_code)
            .ok()
    }
}

impl OpenApiResponderInner for SimulationError {
    fn responses(_gen: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        Ok(Responses::default())
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
    #[openapi(skip)]
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

#[openapi(skip)]
#[ doc = "Redirects to /api" ]
#[get("/", format = "text/html")]
pub async fn get_root() -> Redirect {
    Redirect::to(uri!(get_api))
}

create_endpoint_with_doc!(
    #[describe("Show details for a simulation")]
    #[example("curl -X GET -H 'Accept: application/json' http://localhost:8000/simulation/1")]
    #[openapi]
    #[get("/simulation/<id>", format="application/json")]
    pub async fn get_simulation_id(id: u64) -> SimulationResult {
        match db::read_simulation(id) {
            Ok(sim) => Ok(Json(sim)),
            Err(e) =>  Err( SimulationError { err: e.to_string(), http_status_code: Status::UnprocessableEntity } )
        }
    }
);

create_endpoint_with_doc!(
    #[describe("List the simulations")]
    #[example("curl -X GET -H 'Accept: application/json' http://localhost:8000/simulation")]
    #[openapi]
    #[get("/simulation", format="application/json")]
    pub async fn get_simulations() -> SimulationResult {
        match db::write_u64(&String::from("redis_key"), 36u64) {
            Ok(_) => (),
            Err(e) =>  return Err( SimulationError { err: format!("Could not write to redis DB: {}", e), http_status_code: Status::Unauthorized } )
        };
        match db::read_u64(&String::from("redis_key")) {
            Ok(i) => Ok(Json(Simulation {
                error: "".to_string(),
                load_profile_data: [].to_vec(),
                model_id: i,
                simulation_id: 1,
                simulation_type: SimulationType::Powerflow
            })),
            Err(e) => Err( SimulationError { err: format!("Could not read from redis DB: {}", e), http_status_code: Status::UnprocessableEntity} )
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
    pub async fn post_simulation(form: Form <SimulationFormStruct < '_ >> ) -> Result<Json<Simulation>, SimulationError> {
        match parse_simulation_form(form).await {
            Ok(simulation) => {
                match block_on(amqp::publish(&simulation)) {
                    Ok(()) => Ok(simulation),
                    Err(e) => Err(SimulationError  {
                                        err: format!("Could not publish to amqp server: {}", e),
                                        http_status_code: Status::BadGateway
                                  })
                }
            },
            Err(e) => Err(SimulationError  {
                                err: format!("Error when parsing form: {}", e),
                                http_status_code: Status::NotAcceptable
                          })
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

#[openapi(skip)]
#[get("/swagger.html")]
pub async fn swagger_endpoint() -> (ContentType, &'static str) {
    (ContentType::HTML, "<!DOCTYPE html>
    <html>
     <head>
        <link rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/swagger-ui-dist@3.17.0/swagger-ui.css\">
        <script src=\"//unpkg.com/swagger-ui-dist@3/swagger-ui-bundle.js\"></script>
        <script>

            function render() {
                var ui = SwaggerUIBundle({
                    url:  `/openapi.json`,
                    dom_id: '#swagger-ui',
                    presets: [
                        SwaggerUIBundle.presets.apis,
                        SwaggerUIBundle.SwaggerUIStandalonePreset
                    ]
                });
            }

        </script>
    </head>

    <body onload=\"render()\">
        <div id=\"swagger-ui\"></div>
    </body>
    </html>")
}

#[doc = "Returns the list of routes that we have defined"]
pub fn get_routes() -> Vec<rocket::Route>{
    return rocket_okapi::openapi_get_routes![ swagger_endpoint, get_root, get_api, get_simulations, post_simulation, get_simulation_id]
}
