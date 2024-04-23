use crate::*;

/// Init the on-chain storage versions of pallets added to the runtime prior to this being an automatic process.
pub struct Migration;

impl OnRuntimeUpgrade for Migration {
    fn on_runtime_upgrade() -> Weight {
        use frame_support::traits::GetStorageVersion;
        use frame_support::traits::StorageVersion;

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
