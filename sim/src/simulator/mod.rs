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

use serde::{Deserialize, Serialize};

use crate::input_modeling::dyn_rng;
use crate::input_modeling::dynamic_rng::SimulationRng;
use crate::models::{DevsModel, Model, ModelMessage, ModelRecord, Reportable};
use crate::utils::errors::SimulationError;
use crate::utils::set_panic_hook;

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
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Simulation {
    models: Vec<Model>,
    connectors: Vec<Connector>,
    messages: Vec<Message>,
    services: Services,
}

impl Simulation {
    /// This constructor method creates a simulation from a supplied
    /// configuration (models and connectors).
    pub fn post(models: Vec<Model>, connectors: Vec<Connector>) -> Self {
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
        models: Vec<Model>,
        connectors: Vec<Connector>,
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
    pub fn put(&mut self, models: Vec<Model>, connectors: Vec<Connector>) {
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
    pub fn get_messages(&self) -> &Vec<Message> {
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
            .iter()
            .find(|model| model.id() == model_id)
            .ok_or(SimulationError::ModelNotFound)?
            .status())
    }

    /// This method provides a mechanism for getting the records of any model
    /// in a simulation.  The method takes the model ID as an argument, and
    /// returns the records for that model.
    pub fn get_records(&self, model_id: &str) -> Result<&Vec<ModelRecord>, SimulationError> {
        Ok(self
            .models
            .iter()
            .find(|model| model.id() == model_id)
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
    pub fn models(&self) -> &[Model] {
        &self.models
    }

    pub fn connectors(&self) -> &[Connector] {
        &self.connectors
    }

    /// This method provides a convenient foundation for operating on the
    /// full set of models in the simulation.
    pub fn models_mut(&mut self) -> Vec<&mut Model> {
        self.models.iter_mut().collect()
    }

    /// This method constructs a list of target IDs for a given source model
    /// ID and port.  This message target information is derived from the
    /// connectors configuration.
    fn get_message_target_ids(&self, source_id: &str, source_port: &str) -> Vec<String> {
        self.connectors
            .iter()
            .filter_map(|connector| {
                if connector.source_id() == source_id && connector.source_port() == source_port {
                    Some(connector.target_id().to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// This method constructs a list of target ports for a given source model
    /// ID and port.  This message target information is derived from the
    /// connectors configuration.
    fn get_message_target_ports(&self, source_id: &str, source_port: &str) -> Vec<String> {
        self.connectors
            .iter()
            .filter_map(|connector| {
                if connector.source_id() == source_id && connector.source_port() == source_port {
                    Some(connector.target_port().to_string())
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
        // until_next_event = self.models().iter().fold(INFINITY, |min, model| {
        //     f64::min(min, model.until_next_event())
        // })
        self.models
            .iter()
            .map(|model| model.until_next_event())
            .reduce(f64::min)
            .unwrap()
            .min(f64::INFINITY)
    }

    /// advance the time for all models in the simulation
    pub fn time_advance(&mut self, time_delta: f64) {
        self.models_mut()
            .iter_mut()
            .for_each(|model| model.time_advance(time_delta))
    }

    pub fn run_message_on_model(&mut self, msg: &Message) -> Result<(), SimulationError> {
        // I'm hoping that a copy of these services preserves the clock and random generator state.
        // I think it does.  As they are reference counted.
        let mut services = self.services.clone();
        self.models.iter_mut().try_for_each(|m| {
            match m.id() == msg.target_id() {
                true =>m.events_ext(&ModelMessage {
                    port_name: msg.target_port().to_string(),
                    content: msg.content().to_string(),
                }, &mut services),
                false => Ok(())
            }})
    }

    pub fn handle_messages(&mut self) -> Result<(), SimulationError> {
        self.messages.clone().iter()
            .try_for_each(|msg| self.run_message_on_model(msg))
    }


    /// The simulation step is foundational for a discrete event simulation.
    /// This method executes a single discrete event simulation step,
    /// including internal state transitions, external state transitions,
    /// message orchestration, global time accounting, and step messages
    /// output.
    pub fn step(&mut self) -> Result<Vec<Message>, SimulationError> {
        let mut next_messages: Vec<Message> = Vec::new();
        // Process external events
        &self.handle_messages()?;
        // want a zip between model and messages.
        // Want a model here rather than index to model.
        // (0..self.models.len()).try_for_each(|model_index| -> Result<(), SimulationError> {
        //     // Collect up all the messages that target the model identified by model_index.
        //
        //     // I don't like this because of the clone can't do that
        //     let mm = self.models_mut()[model_index].clone();
        //     let model_messages: Vec<ModelMessage> = self.messages_for_model(&mm);
        //
        //     model_messages
        //         .iter()
        //         .try_for_each(|model_message| -> Result<(), SimulationError> {
        //             self.models[model_index].events_ext(model_message, &mut self.services)
        //         })
        // })?;
    // }


    // Process internal events and gather associated messages
        let until_next_event: f64 = match self.messages.is_empty() {
            true => self.until_next_event(),
            _ => 0.0f64,
        };
        self.time_advance(until_next_event);

        self.services
            .set_global_time(self.services.global_time() + until_next_event);

        let errors: Result<Vec<()>, SimulationError> = (0..self.models.len())
            .map(|model_index| -> Result<(), SimulationError> {
                if self.models[model_index].until_next_event() == 0.0 {
                    self.models[model_index]
                        .events_int(&mut self.services)?
                        .iter()
                        .for_each(|outgoing_message| {
                            let target_ids = self.get_message_target_ids(
                                self.models[model_index].id(), // Outgoing message source model ID
                                &outgoing_message.port_name,   // Outgoing message source model port
                            );
                            let target_ports = self.get_message_target_ports(
                                self.models[model_index].id(), // Outgoing message source model ID
                                &outgoing_message.port_name,   // Outgoing message source model port
                            );
                            target_ids.iter().zip(target_ports.iter()).for_each(
                                |(target_id, target_port)| {
                                    next_messages.push(Message::new(
                                        self.models[model_index].id().to_string(),
                                        outgoing_message.port_name.clone(),
                                        target_id.clone(),
                                        target_port.clone(),
                                        self.services.global_time(),
                                        outgoing_message.content.clone(),
                                    ));
                                },
                            );
                        });
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
    pub fn step_until(&mut self, until: f64) -> Result<Vec<Message>, SimulationError> {
        let mut message_records: Vec<Message> = Vec::new();
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
    pub fn step_n(&mut self, n: usize) -> Result<Vec<Message>, SimulationError> {
        let mut message_records: Vec<Message> = Vec::new();
        (0..n)
            .map(|_| -> Result<Vec<Message>, SimulationError> {
                self.step()?;
                message_records.extend(self.messages.clone());
                Ok(Vec::new())
            })
            .find(Result::is_err)
            .unwrap_or(Ok(message_records))
    }
}
