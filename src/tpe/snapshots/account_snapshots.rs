use super::AccountSnapshot;

use crate::ids::ClientId;
use crate::AccountReport;
use crate::Result;

use std::collections::HashMap;

/// Convenience structure for mapping client IDs to Account snapshots
#[derive(Debug, Default)]
pub struct AccountSnapshots {
    map: HashMap<ClientId, AccountSnapshot>,
}

impl AccountSnapshots {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_mut_or_create(&mut self, client_id: ClientId) -> &mut AccountSnapshot {
        self.map
            .entry(client_id)
            .or_insert_with(|| AccountSnapshot::new(client_id));

        return self.map.get_mut(&client_id).unwrap();
    }

    pub fn build_report(&self) -> Result<Vec<AccountReport>> {
        return self
            .map
            .values()
            .map(|snapshot| snapshot.parse_report())
            .collect::<Result<Vec<AccountReport>>>();
    }
}
