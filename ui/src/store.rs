use leptos::prelude::*;

use crate::types::{Organization, ReportingPeriod};

#[derive(Clone, Copy)]
pub struct AppStore {
    pub active_org: RwSignal<Option<Organization>>,
    pub active_period: RwSignal<Option<ReportingPeriod>>,
    pub current_route: RwSignal<String>,
}

impl AppStore {
    pub fn new() -> Self {
        Self {
            active_org: RwSignal::new(None),
            active_period: RwSignal::new(None),
            current_route: RwSignal::new("/".to_string()),
        }
    }

    pub fn navigate(&self, path: impl Into<String>) {
        self.current_route.set(path.into());
    }
}
