use std::ops::Deref;
use sim::checker::Checker;
use sim::input_modeling::ContinuousRandomVariable;
use sim::models::{DevsModel, Generator, Model, Processor, Storage, ModelMessage};
use sim::simulator::{Connector, Services, Simulation};

fn sample_gps_models() -> [Model; 3]{
    [
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
    ]
}
fn sample_gps_models_dup() -> [Model; 5] {
    [
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
    let mut sim = Simulation::post(sample_gps_models().to_vec(), [].to_vec());
    assert_eq!(sim.unique_model_ids().unwrap(), ());

    let mut sim = Simulation::post(sample_gps_models_dup().to_vec(), [].to_vec());
    assert_eq!(sim.unique_model_ids().unwrap_err(), "Duplicate Model ids found: generator-01, storage-01".to_string());
}

#[test]
fn checker_duplicate_models_hash()
{
    let mut sim = Simulation::post(sample_gps_models().to_vec(), [].to_vec());
    let mut mhm = sim.model_hash_try();
    assert_eq!(mhm.unwrap().len(), 3);

    let sim = Simulation::post(sample_gps_models_dup().to_vec(), [].to_vec());
    let mhm = sim.model_hash_try();
    match mhm {
        Err(error_str) => assert_eq!(error_str, "Model with id generator-01 already exists"),
        Ok(_) => assert!(false),
    }
}

#[test]
fn checker_connector_models_match()
{
    let mut sim = Simulation::post(sample_gps_models().to_vec(), sample_gps_connectors().to_vec());
    let mut chc = sim.connectors_source_to_model();
    assert!(chc.is_ok());
    let mut chc = sim.connectors_target_to_model();
    assert!(chc.is_ok());

    let mut sim = Simulation::post(sample_gps_models().to_vec(), sample_gps_connectors_bad().to_vec());
    let mut chc = sim.connectors_source_to_model();
    assert!(chc.is_err());
    match chc {
        Err(error_str) => assert_eq!(error_str, "Connector connector-fake model not found with source_id processor-99".to_string()),
        Ok(_) => assert!(false),
    }
    let mut chc = sim.connectors_target_to_model();
    match chc {
        Err(error_str) => assert_eq!(error_str, "Connector connector-fake model not found with target_id processor-99".to_string()),
        Ok(_) => assert!(false),
    }
}