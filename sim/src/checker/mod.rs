use crate::models::{DevsModel, Model, ModelMessage};
use crate::simulator::{Services, Simulation};
use std::collections::HashMap;
use itertools::Itertools;

/// Provide tools to check a simulation and verify that the
/// models and connections within it are 'correct'
///
///

pub trait Checker {
    fn model_hash_try<'a>(&'a self) -> Result<HashMap<&'a str, &'a Model>, String>;
    fn unique_model_ids(&self) -> Result<(), String>;

    fn connectors_source_to_model(&self) -> Result<(), String>;
    fn connectors_target_to_model(&self) -> Result<(), String>;
}

impl Checker for Simulation {
    /// build a hash containing the model id and model.  Result is returned so that a duplicate
    /// model id will be identified as an error condition.
    fn model_hash_try(&self) -> Result<HashMap<&str, &Model>, String> {
        let mut models_by_id = HashMap::with_capacity(self.models().len());
        match self.models().iter().try_for_each(|model| {
            match models_by_id.contains_key(model.id()) {
                true => return Err(format!("Model with id {} already exists", model.id())),
                false => match models_by_id.insert(model.id(), model) {
                    Some(_) => {
                        return Err(format!(
                            "Model hash insert failed for key {}.  Unexpected.",
                            model.id()
                        ))
                    }
                    None => Ok(()),
                },
            }
        }) {
            Ok(()) => Ok(models_by_id),
            Err(e) => Err(e.to_string()),
        }
    }

    fn unique_model_ids(&self) -> Result<(), String> {
        let dups: Vec<&str> = self.models().iter().duplicates_by(|model| model.id()).map(|mdl| mdl.id()).collect();
        match dups.len()
        {
            0 => Ok(()),
            _ => Err(format!("Duplicate Model ids found: {}", dups.join(", "))),

        }
    }

    fn connectors_source_to_model(&self) -> Result<(), String> {
        let model_hash = self.model_hash_try().unwrap();
        self.connectors().iter()
            .try_for_each(|connector| match model_hash.get(connector.source_id()){
                Some(_) => Ok(()),
                None => Err(format!("Connector {} model not found with source_id {}", connector.id(), connector.source_id())),
            })
    }

    fn connectors_target_to_model(&self) -> Result<(), String> {
        let model_hash = self.model_hash_try().unwrap();
        self.connectors().iter()
            .try_for_each(|connector| match model_hash.get(connector.target_id()){
                Some(_) => Ok(()),
                None => Err(format!("Connector {} model not found with target_id {}", connector.id(), connector.source_id())),
            })
    }
}
