use crate::models::Model;
use crate::simulator::Simulation;
use itertools::Itertools;
use std::collections::HashMap;

/// Provide tools to check a simulation and verify that the
/// models and connections within it are 'correct'
///
///

pub trait Checker {
    fn model_hash_try<'a>(&'a self) -> Result<HashMap<&'a str, &'a Model>, String>;
    fn unique_model_ids(&self) -> Result<(), String>;

    fn connectors_source_to_model(&self) -> Result<(), String>;
    fn connectors_target_to_model(&self) -> Result<(), String>;

    fn valid_messages(&self) -> Result<(), String>;

    fn check(&self) -> Result<(), String>;
}

impl Checker for Simulation {
    fn check(&self) -> Result<(), String> {
        //Check all of the contained checks.  if any return an error result then bail.
        //was hoping I could do something fancy with method pointers, but not so luck...
        let checks = &[
            self.unique_model_ids(),
            self.connectors_source_to_model(),
            self.connectors_target_to_model(),
            self.valid_messages(),
        ];

        checks.iter().try_for_each(|rslt| rslt.clone())
    }

    /// build a hash containing the model id and model.  Result is returned so that a duplicate
    /// model id will be identified as an error condition.
    fn model_hash_try(&self) -> Result<HashMap<&str, &Model>, String> {
        let mut models_by_id = HashMap::with_capacity(self.get_models().len());
        match self.get_models().iter().try_for_each(|model| {
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
        let dups: Vec<&str> = self
            .get_models()
            .iter()
            .duplicates_by(|model| model.id())
            .map(|mdl| mdl.id())
            .collect();
        match dups.len() {
            0 => Ok(()),
            _ => Err(format!("Duplicate Model ids found: {}", dups.join(", "))),
        }
    }

    fn connectors_source_to_model(&self) -> Result<(), String> {
        let model_hash = self.model_hash_try().unwrap();
        self.get_connectors().iter().try_for_each(|connector| {
            match model_hash.get(connector.source_id()) {
                Some(_) => Ok(()),
                None => Err(format!(
                    "Connector {} model not found with source_id {}",
                    connector.id(),
                    connector.source_id()
                )),
            }
        })
    }

    fn connectors_target_to_model(&self) -> Result<(), String> {
        let model_hash = self.model_hash_try().unwrap();
        self.get_connectors().iter().try_for_each(|connector| {
            match model_hash.get(connector.target_id()) {
                Some(_) => Ok(()),
                None => Err(format!(
                    "Connector {} model not found with target_id {}",
                    connector.id(),
                    connector.source_id()
                )),
            }
        })
    }

    //TODO any initial messages should have a target_id that matches a model node.
    fn valid_messages(&self) -> Result<(), String> {
        let model_hash = self.model_hash_try().unwrap();
        self.get_messages()
            .iter()
            .enumerate()
            .try_for_each(
                |(index, connector)| match model_hash.get(connector.target_id()) {
                    Some(_) => Ok(()),
                    None => Err(format!(
                        "Pending message {} with target id model '{}' not found",
                        index,
                        connector.target_id()
                    )),
                },
            )
    }
}
