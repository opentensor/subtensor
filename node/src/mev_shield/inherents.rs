use super::ShieldKeystore;
use sp_inherents::{Error, InherentData, InherentIdentifier};
use std::sync::Arc;

// The inherent identifier for the next MEV-Shield public key.
pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"shieldpk";

/// The inherent type for the next MEV-Shield public key.
pub type InherentType = Vec<u8>;

pub struct InherentDataProvider {
    keystore: Arc<ShieldKeystore>,
}

impl InherentDataProvider {
    pub fn new(keystore: Arc<ShieldKeystore>) -> Self {
        Self { keystore }
    }
}

#[async_trait::async_trait]
impl sp_inherents::InherentDataProvider for InherentDataProvider {
    async fn provide_inherent_data(&self, inherent_data: &mut InherentData) -> Result<(), Error> {
        let public_key = self.keystore.next_public_key().ok().unwrap_or_default();
        inherent_data.put_data(INHERENT_IDENTIFIER, &public_key)
    }

    async fn try_handle_error(
        &self,
        _: &InherentIdentifier,
        _: &[u8],
    ) -> Option<Result<(), Error>> {
        None
    }
}
