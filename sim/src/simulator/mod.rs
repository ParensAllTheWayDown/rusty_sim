//! The simulator module provides the mechanics to orchestrate the models and
//! connectors via discrete event simulation.  The specific formalism for
//! simulation execution is the Discrete Event System Specification.  User
//! interaction is also captured in this module - simulation stepping and
//! input injection.
//!
//! `Simulation` and `WebSimulation` are used for Rust- and npm-based
//! projects, respectively.  The `Simulation` methods use the associated
//! struct types directly, while the `WebSimulation` provides an interface
//! with better JS/WASM compatibility.
//!
//! Most simulation analysis will involve the collection, transformation,
//! and analysis of messages.  The `step`, `step_n`, and `step_until` methods
//! return the messages generated during the execution of the simulation
//! step(s), for use in message analysis.

use crate::input_modeling::dyn_rng;
use crate::input_modeling::dynamic_rng::SimulationRng;
use crate::models::{DevsModel, Model, ModelMessage, ModelRecord, Reportable};
use crate::utils::errors::{SimulationError, SimulationResult};
use crate::utils::set_panic_hook;
use itertools::{process_results, Itertools};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod coupling;
pub mod services;
pub mod web;

pub use self::coupling::{Connector, Message};
pub use self::services::Services;
pub use self::web::Simulation as WebSimulation;

/// The `Simulation` struct is the core of sim, and includes everything
/// needed to run a simulation - models, connectors, and a random number
/// generator.  State information, specifically global time and active
/// messages are additionally retained in the struct.
///
pub type ModelCollectionType = HashMap<String, Model>;
pub type ConnectorCollectionType = Vec<Connector>;
pub type MessageCollectionType = Vec<Message>;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Simulation {
    models: ModelCollectionType,
    connectors: ConnectorCollectionType,
    messages: MessageCollectionType,
    services: Services,
}

impl Simulation {
    /// This constructor method creates a simulation from a supplied
    /// configuration (models and connectors).
    pub fn post(models: Vec<Model>, connectors: ConnectorCollectionType) -> Self {
        set_panic_hook();
        Self {
            models: models
                .iter()
                .map(|m| (m.id().to_string(), m.clone()))
                .collect(),
            connectors,
            ..Self::default()
        }
    }

    pub fn post_hd(models: ModelCollectionType, connectors: ConnectorCollectionType) -> Self {
        set_panic_hook();
        Self {
            models,
            connectors,
            ..Self::default()
        }
    }

    /// This constructor method creates a simulation from a supplied
    /// configuration (models and connectors).
    pub fn post_with_rng(
        models: ModelCollectionType,
        connectors: ConnectorCollectionType,
        global_rng: impl SimulationRng + 'static,
    ) -> Self {
        set_panic_hook();
        Self {
            models,
            connectors,
            services: Services {
                global_rng: dyn_rng(global_rng),
                global_time: 0.0,
            },
            ..Self::default()
        }
    }

    pub fn set_rng(&mut self, rng: impl SimulationRng + 'static) {
        self.services.global_rng = dyn_rng(rng)
    }

    /// This method sets the models and connectors of an existing simulation.
    pub fn put(&mut self, models: Vec<Model>, connectors: ConnectorCollectionType) {
        self.models = models
            .iter()
            .map(|m| (m.id().to_string(), m.clone()))
            .collect();
        self.connectors = connectors;
    }

    pub fn put_hm(&mut self, models: ModelCollectionType, connectors: ConnectorCollectionType) {
        self.models = models;
        self.connectors = connectors;
    }

    /// Simulation steps generate messages, which are then consumed on
    /// subsequent simulation steps.  These messages between models in a
    /// simulation drive much of the discovery, analysis, and design.  This
    /// accessor method provides the list of active messages, at the current
    /// point of time in the simulation.  Message history is not retained, so
    /// simulation products and projects should collect messages as needed
    /// throughout the simulation execution.
    pub fn get_messages(&self) -> &MessageCollectionType {
        &self.messages
    }

    /// An accessor method for the simulation global time.
    pub fn get_global_time(&self) -> f64 {
        self.services.global_time()
    }

    /// This method provides a mechanism for getting the status of any model
    /// in a simulation.  The method takes the model ID as an argument, and
    /// returns the current status string for that model.
    pub fn get_status(&self, model_id: &str) -> Result<String, SimulationError> {
        Ok(self
            .models
            .get(model_id)
            .ok_or(SimulationError::ModelNotFound)?
            .status())
    }

    /// This method provides a mechanism for getting the records of any model
    /// in a simulation.  The method takes the model ID as an argument, and
    /// returns the records for that model.
    pub fn get_records(&self, model_id: &str) -> Result<&Vec<ModelRecord>, SimulationError> {
        Ok(self
            .models
            .get(model_id)
            .ok_or(SimulationError::ModelNotFound)?
            .records())
    }

    /// To enable simulation replications, the reset method resets the state
    /// of the simulation, except for the random number generator.
    /// Recreating a simulation from scratch for additional replications
    /// does not work, due to the random number generator seeding.
    pub fn reset(&mut self) {
        self.reset_messages();
        self.reset_global_time();
    }

    /// Clear the active messages in a simulation.
    pub fn reset_messages(&mut self) {
        self.messages = Vec::new();
    }

    /// Reset the simulation global time to 0.0.
    pub fn reset_global_time(&mut self) {
        self.services.set_global_time(0.0);
    }

    /// Provide immutable reference to models for analysis.  Can't change.  Just look.
    pub fn get_models(&self) -> &ModelCollectionType {
        &self.models
    }

    pub fn get_model_mut(&mut self, model_id: &str) -> SimulationResult<&mut Model> {
        self.models
            .get_mut(model_id)
            .ok_or(SimulationError::ModelNotFound)
    }

    pub fn get_connectors(&self) -> &[Connector] {
        &self.connectors
    }

    fn get_message_target_tuple(
        &self,
        source_id: &str,
        source_port: &str,
    ) -> Vec<(String, String)> {
        self.connectors
            .iter()
            .filter_map(|connector| {
                if connector.source_id() == source_id && connector.source_port() == source_port {
                    Some((
                        connector.target_id().to_string(),
                        connector.target_port().to_string(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Input injection creates a message during simulation execution,
    /// without needing to create that message through the standard
    /// simulation constructs.  This enables live simulation interaction,
    /// disruption, and manipulation - all through the standard simulation
    /// message system.
    pub fn inject_input(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Calculate the minimum next event time of all models in simulation.
    pub fn until_next_event(&self) -> f64 {
        self.models
            .iter()
            .map(|(id, model)| model.until_next_event())
            .reduce(f64::min)
            .unwrap()
            .min(f64::INFINITY)
    }

    /// advance the time for all models in the simulation
    pub fn time_advance(&mut self, time_delta: f64) {
        self.models
            .iter_mut()
            .for_each(|(id, model)| model.time_advance(time_delta))
    }

    pub fn handle_messages(&mut self, messages: MessageCollectionType) -> SimulationResult<()> {
        messages.iter().try_for_each(|msg| {
            let mut services = self.services.clone();
            let mut model = self.get_model_mut(msg.target_id())?;
            model.events_ext(
                &ModelMessage {
                    port_name: msg.target_port().to_string(),
                    content: msg.content().to_string(),
                },
                &mut services,
            )
        })
    }

    /// for each new message,
    /// locate the target model information for the message.  There may be multiple targets.
    /// for each target create a new Message
    pub fn map_new_messages(
        &mut self,
        model_id: String,
        new_messages: &Vec<ModelMessage>,
    ) -> MessageCollectionType {
        let result_messages: MessageCollectionType = new_messages
            .iter()
            .flat_map(|outgoing_message| self.map_new_message(model_id.clone(), outgoing_message))
            .collect();
        result_messages
    }

    pub fn map_new_message(
        &mut self,
        model_id: String,
        new_message: &ModelMessage,
    ) -> MessageCollectionType {
        self.get_message_target_tuple(&*model_id, &*new_message.port_name)
            .iter()
            .map(|(target_id, target_port)| {
                Message::new(
                    model_id.to_string(),
                    new_message.port_name.clone(),
                    target_id.clone(),
                    target_port.clone(),
                    self.services.global_time(),
                    new_message.content.clone(),
                )
            })
            .collect()
    }

    /// The simulation step is foundational for a discrete event simulation.
    /// This method executes a single discrete event simulation step,
    /// including internal state transitions, external state transitions,
    /// message orchestration, global time accounting, and step messages
    /// output.
    pub fn step(&mut self) -> SimulationResult<MessageCollectionType> {
        let mut next_messages: MessageCollectionType = Vec::new();
        // Process external events
        &self.handle_messages(self.messages.clone())?;

        // Process internal events and gather associated messages
        let until_next_event: f64 = match self.messages.is_empty() {
            true => self.until_next_event(),
            _ => 0.0f64,
        };
        &self.time_advance(until_next_event);

        &self
            .services
            .set_global_time(self.services.global_time() + until_next_event);

        // Not going to add a new model while stepping
        let model_id_cold = &self.models.keys().map(|id| id.clone()).collect_vec();

        let errors: Result<Vec<()>, SimulationError> = model_id_cold
            .iter()
            .map(|model_index| -> SimulationResult<()> {
                // before a change happens on model calculate if the model has an event that is due now.
                let model_cold = self
                    .models
                    .get(&*model_index)
                    .ok_or(SimulationError::ModelNotFound)?;
                // models filtered to those with eminent next event time.
                match model_cold.until_next_event() == 0.0 {
                    true => {
                        // Get a mutable reference to the model because `events_int` will cause changes.
                        let mut mmodel = self
                            .models
                            .get_mut(&*model_index)
                            .ok_or(SimulationError::ModelNotFound)?;

                        mmodel
                            .events_int(&mut self.services)?
                            .iter()
                            //Events_int produces a vector of model messages that must be propagated.
                            .for_each(|outgoing_message| {
                                //Using connection information, calculate all the target_id and target_ports
                                //for a model message that was emitted by events_int.
                                //there may be multiple targets so this is a vector.
                                let target_tuple = self.get_message_target_tuple(
                                    model_index,                 // Outgoing message source model ID
                                    &outgoing_message.port_name, // Outgoing message source model port
                                );
                                
                                // we know that next_messages will be grown by each item in target_tuple
                                // so use an extend to add them all.
                                next_messages.extend(
                                    target_tuple
                                        .iter()
                                        // for each target tuple, create a new Message
                                        // and push each message onto the 'next_messages' that will become the
                                        // messages handled in the next step.
                                        .map(|(target_id, target_port)| {
                                            Message::new(
                                                model_index.to_string(),
                                                outgoing_message.port_name.clone(),
                                                target_id.clone(),
                                                target_port.clone(),
                                                self.services.global_time(),
                                                outgoing_message.content.clone(),
                                            )
                                        }),
                                );
                            });
                    }
                    false => {}
                }
                Ok(())
            })
            .collect();
        errors?;
        self.messages = next_messages;
        Ok(self.get_messages().clone())
    }

    /// This method executes simulation `step` calls, until a global time
    /// has been exceeded.  At which point, the messages from all the
    /// simulation steps are returned.
    pub fn step_until(&mut self, until: f64) -> Result<MessageCollectionType, SimulationError> {
        let mut message_records: MessageCollectionType = Vec::new();
        loop {
            self.step()?;
            if self.services.global_time() < until {
                message_records.extend(self.get_messages().clone());
            } else {
                break;
            }
        }
        Ok(message_records)
    }

    /// This method executes the specified number of simulation steps, `n`.
    /// Upon execution of the n steps, the messages from all the steps are
    /// returned.
    pub fn step_n(&mut self, n: usize) -> Result<MessageCollectionType, SimulationError> {
        let mut message_records: MessageCollectionType = Vec::new();
        (0..n)
            .map(|_| -> Result<MessageCollectionType, SimulationError> {
                self.step()?;
                message_records.extend(self.messages.clone());
                Ok(Vec::new())
            })
            .find(Result::is_err)
            .unwrap_or(Ok(message_records))
    }

    //TODO Only collect messages meeting some predicate
    // all the step methods collect all messages this may be a lot of messages
    // if a simulation runs to steady state.
    // It might be good to have a step method that filters the messages that are kept so the caller
    // can determine if they want to keep all or just messages meeting some predicate conditions.
}
