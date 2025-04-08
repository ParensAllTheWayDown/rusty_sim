use std::ops::Deref;
use sim::checker::Checker;
use sim::input_modeling::ContinuousRandomVariable;
use sim::models::{DevsModel, Generator, Model, Processor, Storage, ModelMessage};
use sim::simulator::{Services, Simulation};

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
fn sample_gps_models_dup() -> [Model; 4] {
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
    ]
}



#[test]
fn checker_duplicate_models_ok()
{
    let mut sim = Simulation::post(sample_gps_models().to_vec(), [].to_vec());
    let mut mhm = sim.model_hash();
    assert_eq!(mhm.len(), 3);
}

#[test]
fn checker_duplicate_models_fail()
{
    let sim = Simulation::post(sample_gps_models_dup().to_vec(), [].to_vec());
    let mhm = sim.model_hash_try();
    match mhm {
        Err(error_str) => assert_eq!(error_str, "Model with id generator-01 already exists"),
        Ok(_) => assert!(false),
    }
}