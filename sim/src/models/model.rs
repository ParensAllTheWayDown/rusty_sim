use std::hash::{Hash, Hasher};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::model_trait::{DevsModel, Reportable, ReportableModel, SerializableModel};
use super::{ModelMessage, ModelRecord};
use crate::simulator::Services;
use crate::utils::errors::{SimulationError, SimulationResult};

/// `Model` wraps `model_type` and provides common ID functionality (a struct
/// field and associated accessor method).  The simulator requires all models
/// to have an ID.
#[derive(Clone)]
pub struct Model {
    id: String,
    inner: Box<dyn ReportableModel>,
}

impl Model {
    pub fn new(id: String, inner: Box<dyn ReportableModel>) -> Self {
        Self { id, inner }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }
}

impl Serialize for Model {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let extra_fields: serde_yaml::Value = self.inner.serialize();
        let mut model = serializer.serialize_map(None)?;
        model.serialize_entry("id", &self.id)?;
        model.serialize_entry("type", self.inner.get_type())?;
        if let serde_yaml::Value::Mapping(map) = extra_fields {
            for (key, value) in map.iter() {
                model.serialize_entry(&key, &value)?;
            }
        }
        model.end()
    }
}

impl<'de> Deserialize<'de> for Model {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let model_repr = super::ModelRepr::deserialize(deserializer)?;
        let concrete_model =
            super::model_factory::create::<D>(&model_repr.model_type[..], model_repr.extra)?;
        Ok(Model::new(model_repr.id, concrete_model))
    }
}

impl SerializableModel for Model {}

// Implement equal and hash for Model so that a model can be used as a key in a HashMap.
// It is very important that a model.id is unique in the system.
impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Model {}

impl Hash for Model {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl DevsModel for Model {
    fn events_ext(
        &mut self,
        incoming_message: &ModelMessage,
        services: &mut Services,
    ) -> SimulationResult<()> {
        self.inner.events_ext(incoming_message, services)
    }

    fn events_int(
        &mut self,
        services: &mut Services,
    ) -> SimulationResult<Vec<ModelMessage>> {
        self.inner.events_int(services)
    }

    fn time_advance(&mut self, time_delta: f64) {
        self.inner.time_advance(time_delta);
    }

    fn until_next_event(&self) -> f64 {
        self.inner.until_next_event()
    }

    #[cfg(feature = "simx")]
    fn event_rules_scheduling(&self) -> &str {
        self.inner.event_rules_scheduling()
    }

    #[cfg(feature = "simx")]
    fn event_rules(&self) -> String {
        self.inner.event_rules()
    }
}

impl Reportable for Model {
    fn status(&self) -> String {
        self.inner.status()
    }

    fn records(&self) -> &Vec<ModelRecord> {
        self.inner.records()
    }
}

impl ReportableModel for Model {}

