var searchIndex = JSON.parse('{\
"dpsim_api":{"doc":"DPSIM Service Rest API for controlling DPSim analyzer","t":[13,13,3,3,4,0,11,11,11,11,11,11,11,11,0,11,11,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,12,5,11,11,11,12,12,5,5,11,11,11,11,11,11,5,0,11,11,12,12,12,11,11,11,11,11,11,11,11,11,11,11,11,11,5,5,5,5,5,5,5,3,3,11,11,11,11,11,11,12,14,5,5,11,11,5,5,5,5,5,5,12,5,11,11,11,11,12,11,11,12,12,12,5,12,11,11,11,11,11,11,11,11,11,11,11],"n":["Outage","Powerflow","Simulation","SimulationForm","SimulationType","amqp","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","clone","clone_into","db","deserialize","deserialize","error","finalize","finalize","finalize","fmt","fmt","from","from","from","from_value","init","init","init","into","into","into","into_collection","into_collection","into_collection","load_profile_data","load_profile_data","main","mapped","mapped","mapped","model_id","model_id","osstr_to_string","parse_simulation_form","push_data","push_data","push_data","push_value","push_value","push_value","read_zip","routes","serialize","serialize","simulation_id","simulation_type","simulation_type","to_owned","try_from","try_from","try_from","try_into","try_into","try_into","type_id","type_id","type_id","vzip","vzip","vzip","publish","get_connection","get_new_simulation_id","read_simulation","read_u64","write_simulation","write_u64","Route","RoutesContext","borrow","borrow","borrow_mut","borrow_mut","clone","clone_into","collapse_id","create_endpoint_with_doc","document_link","document_link_page","from","from","get_api","get_post_simulation","get_root","get_routes","get_simulation_id","get_simulations","heading_id","incomplete_form","into","into","into_collection","into_collection","link","mapped","mapped","method","name","path","post_simulation","routes","serialize","serialize","to_owned","try_from","try_from","try_into","try_into","type_id","type_id","vzip","vzip"],"q":["dpsim_api","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","dpsim_api::amqp","dpsim_api::db","","","","","","dpsim_api::routes","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""],"d":["","","","Form for submitting a new Simulation","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","The main entry point for Rocket","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Function for requesting a new Simulation id from the Redis …","Function for reading a Simulation from a Redis DB","Utility function for reading an int value from a key in a …","Function for writing a Simulation into a Redis DB","Utility function for writing an int value to a key in a …","A struct used for sharing info about a route with a …","A struct used for sharing info about some routes with a …","","","","","","","","","Create a link to the documentation page for the given …","Create an html button linking to the documentation page …","","","List the endpoints","","Redirects to /api","Returns the list of routes that we have defined","Show details for a simulation","List the simulations","","Handler for when an incomplete form has been submitted","","","","","","","","","","","Create a new simulation","","","","","","","","","","","",""],"i":[1,1,0,0,0,0,2,3,1,2,3,1,1,1,0,2,1,2,2,3,1,3,1,2,3,1,1,2,3,1,2,3,1,2,3,1,2,3,0,2,3,1,2,3,0,0,2,3,1,2,3,1,0,0,2,1,2,2,3,1,2,3,1,2,3,1,2,3,1,2,3,1,0,0,0,0,0,0,0,0,0,4,5,4,5,4,4,4,0,0,0,4,5,0,0,0,0,0,0,4,0,4,5,4,5,4,4,5,4,4,4,0,5,4,5,4,4,5,4,5,4,5,4,5],"f":[null,null,null,null,null,null,[[]],[[]],[[]],[[]],[[]],[[]],[[],["simulationtype",4]],[[]],null,[[],["result",4]],[[],["result",4]],null,[[],[["errors",3],["result",4,["errors"]]]],[[],[["errors",3],["result",4,["errors"]]]],[[],[["result",4,["errors"]],["errors",3]]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[]],[[["valuefield",3]],[["errors",3],["result",4,["errors"]]]],[[["options",3]]],[[["options",3]]],[[["options",3]]],[[]],[[]],[[]],[[],["smallvec",3]],[[],["smallvec",3]],[[],["smallvec",3]],null,null,[[],[["result",4,["error"]],["error",3]]],[[],["smallvec",3]],[[],["smallvec",3]],[[],["smallvec",3]],null,null,[[["osstr",3]],["string",3]],[[["form",3,["simulationform"]],["simulationform",3]]],[[["fromformgeneratedcontext",3],["datafield",3]],[["pin",3,["box"]],["box",3,["future"]]]],[[["fromformgeneratedcontext",3],["datafield",3]],[["pin",3,["box"]],["box",3,["future"]]]],[[["fromfieldcontext",3],["datafield",3]],[["box",3,["future","global"]],["pin",3,["box"]]]],[[["valuefield",3]]],[[["valuefield",3]]],[[["valuefield",3]]],[[],[["zipresult",6,["vec"]],["vec",3,["string"]]]],null,[[],["result",4]],[[],["result",4]],null,null,null,[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]],[[]],[[["simulation",3]]],[[],[["redisresult",6,["connection"]],["connection",3]]],[[],[["redisresult",6,["u64"]],["u64",15]]],[[["u64",15]],[["result",4,["simulation","rediserror"]],["simulation",3],["rediserror",3]]],[[["string",3]],[["redisresult",6,["u64"]],["u64",15]]],[[["simulation",3],["string",3]],[["result",4,["rediserror"]],["rediserror",3]]],[[["u64",15],["string",3]],["redisresult",6]],null,null,[[]],[[]],[[]],[[]],[[],["route",3]],[[]],null,null,[[["str",15]],["string",3]],[[["str",15]],["template",3]],[[]],[[]],[[]],[[]],[[]],[[],[["vec",3,["route"]],["route",3]]],[[["u64",15]]],[[]],null,[[["request",3]]],[[]],[[]],[[],["smallvec",3]],[[],["smallvec",3]],null,[[],["smallvec",3]],[[],["smallvec",3]],null,null,null,[[["form",3,["simulationform"]],["simulationform",3]]],null,[[],["result",4]],[[],["result",4]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[]],[[]]],"p":[[4,"SimulationType"],[3,"Simulation"],[3,"SimulationForm"],[3,"Route"],[3,"RoutesContext"]]}\
}');
if (window.initSearch) {window.initSearch(searchIndex)};