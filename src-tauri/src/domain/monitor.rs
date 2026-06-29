use serde::{Deserialize, Serialize};

use super::geometry::WorkArea;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MonitorId(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub id: MonitorId,
    pub name: String,
    pub work_area: WorkArea,
    pub scale_factor: f64,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonitorFallback {
    Primary,
    FirstAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorSelector {
    pub preferred_id: MonitorId,
    pub fallback: MonitorFallback,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonitorSelection<'a> {
    pub monitor: &'a Monitor,
    pub used_fallback: bool,
}

impl MonitorSelector {
    #[must_use]
    pub fn resolve<'a>(&self, monitors: &'a [Monitor]) -> Option<MonitorSelection<'a>> {
        if let Some(monitor) = monitors
            .iter()
            .find(|monitor| monitor.id == self.preferred_id)
        {
            return Some(MonitorSelection {
                monitor,
                used_fallback: false,
            });
        }
        let monitor = match self.fallback {
            MonitorFallback::Primary => monitors.iter().find(|monitor| monitor.is_primary),
            MonitorFallback::FirstAvailable => monitors.first(),
        }
        .or_else(|| monitors.first())?;
        Some(MonitorSelection {
            monitor,
            used_fallback: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Monitor, MonitorFallback, MonitorId, MonitorSelector};
    use crate::domain::geometry::WorkArea;

    fn monitor(id: &str, is_primary: bool) -> Monitor {
        Monitor {
            id: MonitorId(id.to_owned()),
            name: id.to_owned(),
            work_area: WorkArea {
                x: 0,
                y: 0,
                width: 1920,
                height: 1040,
            },
            scale_factor: 1.0,
            is_primary,
        }
    }

    #[test]
    fn selects_the_preferred_monitor() {
        let monitors = [monitor("primary", true), monitor("secondary", false)];
        let selector = MonitorSelector {
            preferred_id: MonitorId("secondary".to_owned()),
            fallback: MonitorFallback::Primary,
        };

        let selection = selector.resolve(&monitors).expect("monitor available");
        assert_eq!(selection.monitor.id.0, "secondary");
        assert!(!selection.used_fallback);
    }

    #[test]
    fn falls_back_to_the_primary_monitor() {
        let monitors = [monitor("secondary", false), monitor("primary", true)];
        let selector = MonitorSelector {
            preferred_id: MonitorId("missing".to_owned()),
            fallback: MonitorFallback::Primary,
        };

        let selection = selector.resolve(&monitors).expect("fallback available");
        assert_eq!(selection.monitor.id.0, "primary");
        assert!(selection.used_fallback);
    }
}
