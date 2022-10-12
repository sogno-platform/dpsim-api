use rocket::response::{self, Redirect, Responder, Response};
use rocket::serde::json::{Json};
use async_global_executor::block_on;
use crate::db;
use crate::amqp;
use crate::amqp::AMQPSimulation;
use rocket_dyn_templates::{Template};
use std::{ str, fmt };
use rocket_okapi::{ openapi, OpenApiError,
                    response::OpenApiResponderInner,
                    gen::OpenApiGenerator };
use okapi::openapi3::Responses;
use serde::{ Serialize, Deserialize };
use rocket::http::{ContentType, Status};
use rocket::{Request};
use schemars::JsonSchema;
use crate::file_service;
use http::uri::InvalidUri as InvalidUri;
use log::info;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[doc = "Struct for encapsulation Simulation details"]
pub struct Simulation {
    pub error: String,
    pub load_profile_id:   String,
    pub model_id:          String,
    pub results_id:        String,
    pub results_data:      String,
    pub simulation_id:     u64,
    pub simulation_type:   SimulationType,
    pub domain:            DomainType,
    pub solver:            SolverType,
    pub timestep:          u64,
    pub finaltime:         u64
}

impl fmt::Display for Simulation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.simulation_id, self.simulation_type, self.model_id)
    }
}

impl fmt::Display for SimulationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.simulation_id, self.simulation_type, self.model_id)
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SimulationSummary {
    pub simulation_id:     u64,
    pub model_id:          String,
    pub simulation_type:   SimulationType,
}

#[doc = "Enum for the various Simulation types"]
#[derive(JsonSchema, FromFormField, Serialize, Deserialize, Debug, Copy, Clone)]
pub enum SimulationType {
    Powerflow,
    Outage
}

impl fmt::Display for SimulationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[doc = "Enum for the various Simulation types"]
#[derive(JsonSchema, FromFormField, Serialize, Deserialize, Debug, Copy, Clone)]
pub enum DomainType {
    SP,
    DP,
    EMT
}

#[doc = "Enum for the various Solver types"]
#[derive(JsonSchema, FromFormField, Serialize, Deserialize, Debug, Copy, Clone)]
pub enum SolverType {
    MNA,
    DAE,
    NRP
}

impl Default for SimulationType {
    fn default() -> Self {
        SimulationType::Powerflow
    }
}

#[doc = "String conversion for the various Simulation types"]
impl SimulationType {
    fn to_string(&self) -> String {
        match &*self {
            SimulationType::Powerflow => "Powerflow".to_owned(),
            SimulationType::Outage    => "Outage".to_owned()
        }
    }
}

/// # Form for submitting a new Simulation
///
/// ## Parameters:
/// * simulation_type
///   - String
///   - must be one of "Powerflow", "Outage"
/// * load_profile_id
///   - String
///   - must be a valid id that exists in the associated sogno file service
/// * model_id
///   - String
///   - must be a valid id that exists in the associated sogno file service
#[derive(FromForm, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SimulationForm {
    pub simulation_type:   SimulationType,
    pub model_id:          String,
    pub load_profile_id:   String,
    #[field(default = DomainType::SP)]
    pub domain:            DomainType,
    #[field(default = SolverType::NRP)]
    pub solver:            SolverType,
    #[field(default = 1)]
    pub timestep:          u64,
    #[field(default = 30)]
    pub finaltime:         u64
}

async fn parse_simulation_form(form: Json<SimulationForm>) -> Result<Json<Simulation>, SimulationError>{
    let simulation_id = match db::get_new_simulation_id() {
        Ok(id) => id,
        Err(e) => return Err(SimulationError {
                             err: format!("Failed to obtain new simulation id: {}", e),
                             http_status_code: Status::BadGateway
                         })
    };
    let results_file     = file_service::create_results_file().await?;
    let simulation = Simulation {
        error:           "".to_string(),
        load_profile_id: form.load_profile_id.clone(),
        model_id:        form.model_id.clone(),
        results_id:      results_file,
        results_data:    "".into(),
        simulation_id:   simulation_id,
        simulation_type: form.simulation_type,
        domain:          form.domain,
        solver:          form.solver,
        timestep:        form.timestep,
        finaltime:       form.finaltime
    };
    match db::write_simulation(&simulation_id.to_string(), &simulation) {
        Ok(()) => Ok(Json(simulation)),
        Err(e) => Err(SimulationError {
                      err: format!("Could not write to db: {}", e.to_string()),
                      http_status_code: Status::BadGateway
                  })
    }
}

macro_rules! create_endpoint_with_doc{
    (
        #[describe($description:tt)]
        #[example($wget:tt)]
        #[summary($sum:tt)]
        #[post($route_path:tt, format = $content_type:tt, data = "<form>")]
        $(#[$attr:meta])*
        pub async fn $name:ident( $($arg_name:ident : $arg_ty:ty),* $(,)? ) $(-> $ret:ty)?
            $body:block
    ) => {
        #[allow(rustdoc::invalid_rust_codeblocks)]
        #[ doc = $sum]
        #[ doc = $description ]
        #[ doc = "\n\r * Content-Type: " ]
        #[ doc = $content_type ]
        #[ doc = "\n\r * Example request:\n\r" ]
        #[ doc = "```" ]
        #[ doc = $wget ]
        #[ doc = "```" ]
        #[openapi]
        #[post($route_path, format = $content_type, data = "<form>")]
        $( #[$attr] )*
        pub async fn $name ( $($arg_name : $arg_ty),* ) $(-> $ret)?
            $body
    };
    (
        #[describe($description:tt)]
        #[example($wget:tt)]
        #[summary($sum:tt)]
        #[$openapi:meta]
        #[get($route_path:tt, format = $content_type:tt)]
        $(#[$attr:meta])*
        pub async fn $name:ident( $($arg_name:ident : $arg_ty:ty),* $(,)? ) $(-> $ret:ty)?
            $body:block
    ) => {
        #[allow(rustdoc::invalid_rust_codeblocks)]
        #[ doc = $sum]
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

impl From<hyper::Error> for SimulationError {
    fn from(input: hyper::Error) -> Self {
        return SimulationError { err: format!("Error converting url: {}", input), http_status_code: rocket::http::Status{ code: 500 } }
    }
}

impl From<InvalidUri> for SimulationError {
    fn from(input: InvalidUri) -> Self {
        return SimulationError { err: format!("Error converting uri: {}", input), http_status_code: rocket::http::Status{ code: 500 } }
    }
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
    #[summary("# SKIPPING OPENAPI GEN")]
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
    #[summary("# Get a simulation using the ID")]
    #[openapi]
    #[get("/simulation/<id>", format="application/json")]
    pub async fn get_simulation_id(id: u64) -> SimulationResult {
        match db::read_simulation(id) {
            Ok(mut sim) => {
                let uri: String = match file_service::convert_id_to_url(&sim.results_id).await {
                    Ok(url) => url,
                    Err(e) => return Err( SimulationError {
                                              err: format!("Could not read convert results id to url. Results id:{} Error: {}", sim.results_id, e),
                                              http_status_code: Status::Unauthorized
                                          })
                };
                let data = file_service::get_data_from_url(&uri).await;
                let results: String = match data {
                    Ok(boxed_data) => {
                        std::str::from_utf8(&boxed_data).unwrap().into()
                    },
                    Err(e) => return Err( SimulationError {
                                              err: format!("Could not read results from url. Results id:{} Error: {}", sim.results_id, e),
                                              http_status_code: Status::Unauthorized
                                          })
                };
                sim.results_data = results;
                Ok(Json(sim))
            },
            Err(e) =>  Err( SimulationError { err: e.to_string(), http_status_code: Status::UnprocessableEntity } )
        }
    }
);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[doc = "Struct for encapsulation Simulation details"]
pub struct SimulationArray {
    pub simulations: Vec<SimulationSummary>
}

create_endpoint_with_doc!(
    #[describe("List the simulations")]
    #[example("curl -X GET -H 'Accept: application/json' http://localhost:8000/simulation")]
    #[summary("# Get array of all simulations")]
    #[openapi]
    #[get("/simulation", format="application/json")]
    pub async fn get_simulations() -> Result<Json<SimulationArray>, SimulationError> {
        match db::get_number_of_simulations() {
            Ok(number_of_simulations) => {
                let mut simvec = Vec::new();
                // "The range start..end contains all values with start <= x < end. It is empty if start >= end."
                // from https://doc.rust-lang.org/std/ops/struct.Range.html
                let last_plus_one = number_of_simulations+1;
                for n in 1..last_plus_one {
                    match db::read_simulation(n) {
                        Ok(sim) => {
                            let sim_summary = SimulationSummary {
                                simulation_id:     sim.simulation_id,
                                model_id:          sim.model_id,
                                simulation_type:   sim.simulation_type,
                            };
                            simvec.push(sim_summary);
                        }
                        Err(e) => return Err( SimulationError { err: format!("Could not read simulation {} from redis DB: {}", n, e), http_status_code: Status::UnprocessableEntity} )
                    }
                }
                let simarray = SimulationArray {
                    simulations: simvec,
                };
                Ok(Json(simarray))
            },
            Err(e) => Err( SimulationError { err: format!("Could not read number of simulations from redis DB: {}", e), http_status_code: Status::UnprocessableEntity} )
        }
    }
);

create_endpoint_with_doc!(
    #[describe("Create a new simulation")]
    #[example("
        curl -X POST
             -H 'Accept: application/json'
             -F file=@testdata/load_profile_data.zip
             -F model_id=\"theModel.zip\"
             -F allowed_file_types=application/zip hocalhost:8000/simulation"
    )]
    #[summary("# Create a simulation")]
    #[post("/simulation", format = "application/json", data = "<form>")]
    pub async fn post_simulation(form: Json<SimulationForm > ) -> SimulationResult {
        match parse_simulation_form(form).await {
            Ok(simulation) => {
                let model_id         = &simulation.model_id;
                let load_profile_id  = &simulation.load_profile_id;
                let model_url        = file_service::convert_id_to_url(model_id).await?;
                let mut load_profile_url = "".into();
                let none_string: String = "None".into();
                info!("load_profile_id: {}", load_profile_id);
                if simulation.load_profile_id != none_string {
                    info!("Converting {} to url", simulation.load_profile_id);
                    load_profile_url = file_service::convert_id_to_url(load_profile_id).await?;
                }
                let amqp_sim         = AMQPSimulation::from_simulation(&simulation, model_url, load_profile_url);
                match block_on(amqp::request_simulation(&amqp_sim)) {
                    Ok(()) => Ok(simulation),
                    Err(e) => Err(SimulationError {
                        err: format!("Could not publish to amqp server: {}", e),
                        http_status_code: Status::BadGateway
                    })
                }
            },
            Err(e) => Err(e)
        }
    }
);

#[doc = "Create a link to the documentation page for the given function"]
fn document_link(fn_name: &str) -> String {
    format!("https://sogno-platform.github.io/dpsim-api/dpsim_api/routes/fn.{}{}", fn_name, ".html")
}

#[doc = "Handler for when an incomplete form has been submitted"]
#[catch(422)]
pub async fn incomplete_form(form: &rocket::Request<'_>) -> String {
    info!("FORM: {}", form);
    format!("Incomplete form.{}", form)
}

#[doc = "Returns the list of routes that we have defined"]
pub fn get_routes() -> Vec<rocket::Route>{
    return rocket_okapi::openapi_get_routes![ get_root, get_api, get_simulations, post_simulation, get_simulation_id]
}
