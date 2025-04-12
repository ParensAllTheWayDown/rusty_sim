use std::collections::HashMap;
use std::ops::Deref;
use sim::checker::Checker;
use sim::input_modeling::ContinuousRandomVariable;
use sim::models::{DevsModel, Generator, Model, Processor, Storage, ModelMessage};
use sim::simulator::{Connector, ModelCollectionType, Services, Simulation};
use sim::utils::errors::SimulationError;

fn sample_gps_models() -> ModelCollectionType {
    let vmod = vec![
        Model::new(
            String::from("generator-01"),
            Box::new(Generator::new(
                ContinuousRandomVariable::Exp { lambda: 0.0957 },
                None,
                String::from("job"),
                false,
                None,
            )),
        ),
        Model::new(
            String::from("processor-01"),
            Box::new(Processor::new(
                ContinuousRandomVariable::Exp { lambda: 0.1659 },
                Some(14),
                String::from("job"),
                String::from("processed"),
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
    ];
    vmod.iter().map(|m|(m.id().to_string(), m.clone())).collect()
}


fn sample_gps_connectors() -> [Connector; 2]{
    [
        Connector::new(
            String::from("connector-01"),
            String::from("generator-01"),
            String::from("processor-01"),
            String::from("job"),
            String::from("job"),
        ),
        Connector::new(
            String::from("connector-02"),
            String::from("processor-01"),
            String::from("storage-01"),
            String::from("processed"),
            String::from("store"),
        ),
    ]
}

fn sample_gps_connectors_bad() -> [Connector; 3]{
    [
        Connector::new(
            String::from("connector-01"),
            String::from("generator-01"),
            String::from("processor-01"),
            String::from("job"),
            String::from("job"),
        ),
        Connector::new(
            String::from("connector-02"),
            String::from("processor-01"),
            String::from("storage-01"),
            String::from("processed"),
            String::from("store"),
        ),
        Connector::new(
            String::from("connector-fake"),
            String::from("processor-99"),
            String::from("storage-88"),
            String::from("processed"),
            String::from("store"),
        ),
    ]
}


#[test]
fn check_duplicate_models() {
    let mut sim = Simulation::post_hd(sample_gps_models(), [].to_vec());
    //should create fine.
}

#[test]
fn checker_duplicate_models_hash()
{
    let mut sim = Simulation::post_hd(sample_gps_models(), [].to_vec());
    assert_eq!(sim.get_models().len(), 3);
}

#[test]
fn checker_connector_models_match()
{
    let mut sim = Simulation::post_hd(sample_gps_models(), sample_gps_connectors().to_vec());
    let mut chc = sim.connectors_source_to_model();
    assert!(chc.is_ok());
    let mut chc = sim.connectors_target_to_model();
    assert!(chc.is_ok());

    let mut sim = Simulation::post_hd(sample_gps_models(), sample_gps_connectors_bad().to_vec());
    let mut chc = sim.connectors_source_to_model();
    assert!(chc.is_err());
    match chc {
        Err(error_str) => assert_eq!(error_str.to_string(), "An invalid model configuration was encountered during simulation"),
        Ok(_) => assert!(false),
    }
    let mut chc = sim.connectors_target_to_model();
    match chc {
        Err(error_str) => assert_eq!(error_str.to_string(), "An invalid model configuration was encountered during simulation"),
        Ok(_) => assert!(false),
    }
}