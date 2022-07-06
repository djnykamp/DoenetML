mod utils;

extern crate web_sys;

use serde_json::json;
use wasm_bindgen::prelude::*;
use serde::Serialize;

use core::ComponentLike;
use core::ComponentSpecificBehavior;
use core::DoenetCore;
use core::create_all_dependencies_for_component;
use core::number;
use core::resolve_state_variable;
use core::state_variable_setup::StateVar;
use std::borrow::Borrow;
use std::clone;
use std::collections::HashMap;

use core::Component;
use core::ComponentChild;

use core::text::Text;
use core::number::Number;

use core::state_variable_setup::{StateVarVariant, Dependency};
use std::hash::Hash;
use std::rc::Rc;

use serde_json::Value;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// A macro to provide println! style syntax for console.log logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}


#[wasm_bindgen]
#[derive(Debug)]
pub struct PublicDoenetCore(DoenetCore);



#[wasm_bindgen]
impl PublicDoenetCore {
    pub fn new(program: &str) -> PublicDoenetCore {

        utils::set_panic_hook();
                
        // log!("core recieved the string: {}", program);

        let json_deserialized: serde_json::Value = serde_json::from_str(program).unwrap();

        log!("rust json: {:#?}", json_deserialized);

        let mut components: HashMap<String, Component> = HashMap::new();
        let mut component_type_counter: HashMap<String, u32> = HashMap::new();

        let mut root_component_name: Option<String> = None;

        add_json_subtree_to_components(
            &mut components,
            &json_deserialized, "",
            &mut component_type_counter,
            &mut root_component_name,
        );

        let root_component_name = root_component_name.unwrap();

        let mut dependencies: Vec<Dependency> = vec![];
        
        for (_, component) in components.iter() {

            let comp = component.component();
            let dependencies_for_this_component = create_all_dependencies_for_component(&comp);

            for dependency in dependencies_for_this_component {
                dependencies.push(dependency);
            }
            
        }


        // Define Doenet Core structure
        let core: DoenetCore = DoenetCore {components, dependencies, root_component_name };




        log!("Components: {:#?}", core.components);
        log!("Dependencies\n{:#?}", core.dependencies);


    
        PublicDoenetCore(core)
        
    }

    pub fn update_renderers(&self) -> String {
        let json_obj = generate_render_tree(&self.0);
        serde_json::to_string(&json_obj).unwrap()
    }
}

fn add_json_subtree_to_components(components: &mut HashMap<String, Component>, json_obj: &serde_json::Value, parent_name: &str, component_type_counter: &mut HashMap<String, u32>, root_component_name: &mut Option<String>) {

    match json_obj {
        serde_json::Value::Array(value_vec) => {
            log!("array");
            for value in value_vec.iter() {
                add_json_subtree_to_components(components, value, parent_name, component_type_counter, root_component_name);
            }
        },
        
        Value::String(string_value) => {
            if let Some(parent) = components.get(parent_name) {
                parent.component().add_as_child(ComponentChild::String(string_value.to_string()));
            }
        },

        serde_json::Value::Object(map) => {
            log!("object");
            let component_type_value = map.get("componentType").unwrap();

            if let Value::String(component_type) = component_type_value {

                let count = *component_type_counter.get(component_type).unwrap_or(&0);
                component_type_counter.insert(component_type.to_string(), count + 1);
                let mut component_name =  format!("/_{}{}", component_type, count + 1);

                let props_value = map.get("props").unwrap();
                if let Value::Object(props_map) = props_value {
                    if let Some(name_value) = props_map.get("name") {
                        if let Value::String(name) = name_value {
                            component_name = name.to_string();
                        }
                    }

                    // Address other props here
                }


                // Make sure to do this before you recurse to the next Value::Object
                if root_component_name.is_none() {
                    *root_component_name = Option::Some(component_name.clone());
                }
                

                let component = match component_type.as_str() {
                    

                    "text" => Component::Text(
                        Text::create(component_name.clone(), parent_name.to_string())
                    ),

                    "number" => Component::Number(
                        Number::create(component_name.clone(), parent_name.to_string())
                    ),

                    // Add components to this match here
 
                    _ => {panic!("Unrecognized component type")}
                };

                if let Some(parent) = components.get(parent_name) {
                    parent.clone().component().add_as_child(
                        ComponentChild::Component(component.clone().component()));
                }

                let children_value = map.get("children").unwrap();

                components.insert(component_name.clone(), component);



                add_json_subtree_to_components(components, children_value, &component_name, component_type_counter, root_component_name);

            } else {
                panic!("componentType is not a string");
            }
        },

        _ => {log!("other");},
    }
}

pub fn generate_render_tree(core: &DoenetCore) -> Value {
    let root_node = core.components.get(&core.root_component_name).unwrap();
    let mut json_obj: Vec<Value> = vec![];
    generate_render_tree_internal(core, &root_node.component(), &mut json_obj);
    json!(json_obj)
}

pub fn generate_render_tree_internal(core: &DoenetCore, component: &Rc<dyn ComponentLike>, json_obj: &mut Vec<Value>) {

    let state_vars = component.state_variable_instructions();

    let renderered_state_vars = state_vars.into_iter().filter(|kv| match kv.1 {
        StateVarVariant::Integer(sv) => sv.for_renderer,
        StateVarVariant::Number(sv) => sv.for_renderer,
        StateVarVariant::String(sv) => sv.for_renderer,
        StateVarVariant::Bool(sv) => sv.for_renderer,
    });

    let mut map = serde_json::Map::new();
    for (name, _variant) in renderered_state_vars {

        resolve_state_variable(core, component, name);
        let state_var_value = component.state_var(name).unwrap();

        map.insert(name.to_string(),
            match state_var_value {
                core::StateVarAccess::Integer(v) => json!(v.borrow().clone()),
                core::StateVarAccess::Number(v) => json!(v.borrow().clone()),
                core::StateVarAccess::String(v) => json!(v.borrow().clone()),
                core::StateVarAccess::Bool(v) => json!(v.borrow().clone()),
            }
        );
    }


    let children_instructions = if component.should_render_children() {
        let children = component.children().borrow();
        children.iter().map(|child| match child {
            ComponentChild::Component(comp) => {
                // recurse for children
                generate_render_tree_internal(core, comp, json_obj);

                json!({
                    "actions": {},
                    "componentName": comp.name().to_string(),
                    "componentType": comp.get_component_type().to_string(),
                    "effectiveName": comp.name().to_string(),
                    "renderType": comp.get_component_type().to_string(),
                })},
            ComponentChild::String(string) => {
                json!({
                    "actions": {},
                    "componentName": string.to_string(),
                    "componentType": "string".to_string(),
                    "effectiveName": string.to_string(),
                    "renderType": "string".to_string(),
                })},
        }).collect()
    } else {
        vec![]
    };

    json_obj.push(json!({
        "componentName": component.name(),
        "stateValues": Value::Object(map),
        "childrenInstructions": json!(children_instructions),
    }));
}
