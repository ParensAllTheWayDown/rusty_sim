use crate::models::Model;
use crate::simulator::Simulation;
use itertools::Itertools;
use std::collections::HashMap;
use crate::utils::errors::{SimulationError, SimulationResult};

/// Provide tools to check a simulation and verify that the
/// models and connections within it are 'correct'
///
///

pub trait Checker {
    fn connectors_source_to_model(&self) -> SimulationResult<()>;
    fn connectors_target_to_model(&self) -> SimulationResult<()>;

    fn valid_messages(&self) -> SimulationResult<()>;

    fn check(&self) -> SimulationResult<()>;
}

impl Checker for Simulation {
    fn check(&self) -> SimulationResult<()> {
        //Check all of the contained checks.  if any return an error result then bail.
        //was hoping I could do something fancy with method pointers, but not so luck...

        //ugh frustrating.  Something like itertools::process_results might work but not going to spend more time on this
        
        let a = self.connectors_source_to_model();
        let b = self.connectors_target_to_model();
        let c = self.valid_messages();
        
        
        match a {
            Ok(_) => match b {
                Ok(_) => match c {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e)
                },
                Err(e) => Err(e)
            },
            Err(e) => Err(e)
        }
    }

    fn connectors_source_to_model(&self) -> SimulationResult<()> {
        self.get_connectors().iter().try_for_each(|connector| {
            match self.get_models().get(connector.source_id()) {
                Some(_) => Ok(()),
                None => Err(SimulationError::InvalidModelConfiguration),
            }
        })
    }

    fn connectors_target_to_model(&self) -> SimulationResult<()> {
        self.get_connectors().iter().try_for_each(|connector| {
            match self.get_models().get(connector.target_id()) {
                Some(_) => Ok(()),
                None => Err(SimulationError::InvalidModelConfiguration),
            }
        })
    }

    //TODO any initial messages should have a target_id that matches a model node.
    fn valid_messages(&self) -> SimulationResult<()> {
        self.get_messages()
            .iter()
            .enumerate()
            .try_for_each(
                |(index, connector)| match self.get_models().get(connector.target_id()) {
                    Some(_) => Ok(()),
                    None => Err(SimulationError::InvalidMessage),
                },
            )
    }
}
