use sp_inherents::{Error, InherentData, InherentIdentifier};
use stp_shield::ShieldKeystorePtr;

// The inherent identifier for the next MEV-Shield public key.
pub const INHERENT_IDENTIFIER: InherentIdentifier = *b"shieldpk";

/// The inherent type for the next MEV-Shield public key.
pub type InherentType = Option<Vec<u8>>;

pub struct InherentDataProvider {
    keystore: ShieldKeystorePtr,
}

impl InherentDataProvider {
    pub fn new(keystore: ShieldKeystorePtr) -> Self {
        Self { keystore }
    }
}

#[async_trait::async_trait]
impl sp_inherents::InherentDataProvider for InherentDataProvider {
    async fn provide_inherent_data(&self, inherent_data: &mut InherentData) -> Result<(), Error> {
        let public_key = self.keystore.next_public_key().ok();
        inherent_data.put_data::<InherentType>(INHERENT_IDENTIFIER, &public_key)
    }

    async fn try_handle_error(
        &self,
        _: &InherentIdentifier,
        _: &[u8],
    ) -> Option<Result<(), Error>> {
        None
    }
}
