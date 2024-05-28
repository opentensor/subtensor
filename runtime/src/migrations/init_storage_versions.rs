use crate::*;

pub mod deprecated_triumvirate_pallet {
    use super::*;

    // Renamed to Governance
    pub type Triumvirate = Governance;
}
/// Init the on-chain storage versions of pallets added to the runtime prior to this being an automatic process.
pub struct Migration;

impl OnRuntimeUpgrade for Migration {
    fn on_runtime_upgrade() -> Weight {
        use frame_support::traits::GetStorageVersion;
        use frame_support::traits::StorageVersion;

        use deprecated_triumvirate_pallet::Triumvirate;

        if Triumvirate::on_chain_storage_version() == StorageVersion::new(0) {
            Triumvirate::current_storage_version().put::<Triumvirate>();
        }
        if TriumvirateMembers::on_chain_storage_version() == StorageVersion::new(0) {
            TriumvirateMembers::current_storage_version().put::<TriumvirateMembers>();
        }
        if SenateMembers::on_chain_storage_version() == StorageVersion::new(0) {
            SenateMembers::current_storage_version().put::<SenateMembers>();
        }
        if Scheduler::on_chain_storage_version() == StorageVersion::new(0) {
            Scheduler::current_storage_version().put::<Scheduler>();
        }

        <Runtime as frame_system::Config>::DbWeight::get().reads_writes(4, 4)
    }
}
