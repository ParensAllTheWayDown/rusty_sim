//Given a collection of models and a collection of messages, map_filter messages to models.

use itertools::iproduct;
use sim::input_modeling::ContinuousRandomVariable;
use sim::models::{Generator, Model, ModelMessage, Storage};
use sim::simulator::{Connector, Message, Simulation};
use sim::utils::errors::SimulationError;

fn base_models() -> Vec<Model> {
    vec![
        Model::new(
            String::from("generator-01"),
            Box::new(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 0.5 },
                None,
                String::from("job"),
                false,
                None,
            )),
        ),
        Model::new(
            String::from("storage-01"),
            Box::new(Storage::new(
                String::from("store"),
                String::from("read"),
                String::from("stored"),
                false,
            )),
        ),
    ]
}

fn base_connectors() -> Vec<Connector> {
    vec![Connector::new(
        String::from("connector-01"),
        String::from("generator-01"),
        String::from("storage-01"),
        String::from("job"),
        String::from("store"),
    )]
}

fn base_messages() -> Vec<Message> {
    vec![
        Message::new(
            "generator-01".to_string(),
            "job".to_string(),
            "storage-01".to_string(),
            "store".to_string(),
            1.0,
            "testing".to_string(),
        ),
        Message::new(
            "generator-01".to_string(),
            "job".to_string(),
            "storage-01".to_string(),
            "store".to_string(),
            1.1,
            "testing 02".to_string(),
        ),
    ]
}

fn base_simulation() -> Simulation {
    let mut sim = Simulation::post(base_models(), base_connectors());
    // Add on message into the model.
    base_messages().iter().for_each(|msg| {
        sim.inject_input(msg.clone());
    });
    sim
}

//Sanity check outside simulation can I do this?

/// iterate over models and find any models that have the desired id. (there should only be one.)
/// clone the model and return a vector of the matched models (but the model is copied)
/// important bit is that I have to specify the type of collection to produce in the collect.
#[test]
fn model_filter_first() {
    let expected_model_id = "generator-01".to_string();
    let models = base_models();
    //get a copy of the model to do the messages_for_model
    let related_models = models
        .iter()
        .filter(|m| m.id() == expected_model_id)
        .map(|om| om.clone())
        .collect::<Vec<_>>();
    assert_eq!(related_models.len(), 1);
    assert_eq!(related_models[0].id(), expected_model_id);
}

/// iterate over models and fine any models with the selected id.
/// return a vector containing mutable model references!
#[test]
fn model_filter() {
    let expected_model_id = "generator-01".to_string();
    let models = base_models();
    let related_models: Vec<&Model> = models
        // .iter_mut()
        .iter()
        .filter(|m| m.id() == expected_model_id)
        .collect::<Vec<_>>();
    assert_eq!(related_models.len(), 1);
    assert_eq!(related_models[0].id(), expected_model_id);
}

///Ah, retain!  Can do this but it mutates the Vec<Models> Which may or may not be ok.
#[test]
fn model_retain() {
    let expected_model_id = "storage-01".to_string();
    let mut models = base_models();
    models.retain(|m| m.id() == expected_model_id);
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id(), expected_model_id);
}
//
// let model = models.iter().filter_map(|m|{
//     match m.id() == "generator-01".to_string() {
//         true => Some(m.clone()),
//         false => None,
//     }
// }).collect().first();

#[test]
fn message_match_filter_map() {
    let expected_model_id = "storage-01".to_string();

    let messages = base_messages();
    let mut models = base_models();
    //for each model, for each message (recycled) where message is addressed to model by target_id.
    //Not efficient because a message is checked for each model even if the message has already been selected for a different model.
    // Honoring the original alg...
    let zmm = iproduct!(messages, models)
        .filter(|(msg, mdl)| {
            println!("{},{}", mdl.id(), msg.target_id());
            mdl.id() == msg.target_id()
        })
        .collect::<Vec<_>>();
    assert_eq!(zmm.len(), 2);
}
#[test]
fn message_to_model_message() {
    let expected_model_id = "storage-01".to_string();

    let messages = base_messages();
    let mut models = base_models();
    //for each model, for each message (recycled) where message is addressed to model by target_id.
    //Not efficient because a message is checked for each model even if the message has already been selected for a different model.
    // Honoring the original alg...
    let zmm = iproduct!(messages, models)
        .filter_map(|(msg, mdl)| {
            println!("{},{}", mdl.id(), msg.target_id());
            match mdl.id() == msg.target_id() {
                true => Some(ModelMessage {
                    port_name: msg.target_port().to_string(),
                    content: msg.content().to_string(),
                }),
                false => None,
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(zmm.len(), 2);
}


// #[test]
// fn mesage_to_model_events_ext() {
//     let sim = base_simulation();
//     iproduct!(sim.messages, sim.models)
//         .filter_map(|(msg, mdl)| {
//             println!("{},{}", mdl.id(), msg.target_id());
//             match mdl.id() == msg.target_id() {
//                 true => Some((mdl, ModelMessage {
//                     port_name: msg.target_port().to_string(),
//                     content: msg.content().to_string(),
//                 })),
//                 false => None,
//             }
//         }).try_for_each(|mdl, mm| -> Result<(), SimulationError> {
//         mdl.events_ext(mm, &mut self.services)
//     })
//         