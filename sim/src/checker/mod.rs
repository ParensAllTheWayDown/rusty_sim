use crate::models::{DevsModel, Model, ModelMessage};
use crate::simulator::{Services, Simulation};
use std::collections::HashMap;

/// Provide tools to check a simulation and verify that the
/// models and connections within it are 'correct'
///
///

pub trait Checker {
    fn unique_model_ids(&self) -> Result<(), String>;
    fn model_hash<'a>(&'a self) -> HashMap<&'a str, &'a Model>;
    fn model_hash_try<'a>(&'a self) -> Result<HashMap<&'a str, &'a Model>, String>;
}

impl Checker for Simulation {
    ///Build a hash containing the model id and a model.  Duplicates are swept under the rug.
    fn model_hash<'a>(&'a self) -> HashMap<&'a str, &'a Model> {
        self.models()
            .iter()
            .map(|model| (model.id(), model))
            .collect::<HashMap<_, _>>()
    }


    /// build a hash containing the model id and model.  Result is returned so that a duplicate
    /// model id will be identified as an error condition.
    fn model_hash_try<'a>(&'a self) -> Result<HashMap<&'a str, &'a Model>, String> {
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
        let model_hash = self.model_hash();

        Ok(())
    }
}
