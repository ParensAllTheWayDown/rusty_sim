//Given a collection of models and a collection of messages, map_filter messages to models.

use itertools::iproduct;
use sim::input_modeling::ContinuousRandomVariable;
use sim::models::{Generator, Model, ModelMessage, Storage};
use sim::simulator::{Connector, Message, ModelCollectionType, Simulation};
use sim::utils::errors::SimulationError;

fn base_models() -> ModelCollectionType {
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
    ].iter().map(|m|(m.id().to_string(), m.clone())).collect()
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
    let mut sim = Simulation::post_hd(base_models(), base_connectors());
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
        .filter(|(model_id, m)| m.id() == expected_model_id)
        .map(|om| om.clone())
        .collect::<Vec<_>>();
    assert_eq!(related_models.len(), 1);
    assert_eq!(*related_models[0].0, expected_model_id);
}

/// iterate over models and fine any models with the selected id.
/// return a vector containing mutable model references!
#[test]
fn model_filter() {
    let expected_model_id = "generator-01".to_string();
    let models = base_models();
    let related_model = models.get(&expected_model_id);
    assert!(related_model.is_some());
}

///Ah, retain!  Can do this but it mutates the Vec<Models> Which may or may not be ok.
#[test]
fn model_retain() {
    let expected_model_id = "storage-01".to_string();
    let mut models = base_models();
    assert!(models.get(&expected_model_id).is_some());
}

