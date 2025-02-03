use super::*;

/// Module containing deprecated storage format for Stake
pub mod deprecated_stake_format {
    use super::*;
    use frame_support::{pallet_prelude::ValueQuery, storage_alias, Blake2_128Concat, Identity};

    #[storage_alias]
    pub(super) type Stake<T: Config> = StorageDoubleMap<
        Pallet<T>,
        Blake2_128Concat,
        <T as frame_system::Config>::AccountId,
        Identity,
        <T as frame_system::Config>::AccountId,
        u64,
        ValueQuery,
    >;
}

pub mod migrate_rao {
    use super::*;
    use codec::MaxEncodedLen;
    use frame_support::{
        migrations::{MigrationId, SteppedMigration, SteppedMigrationError},
        traits::GetStorageVersion,
        weights::WeightMeter,
    };

    #[derive(Decode, Encode, MaxEncodedLen, Eq, PartialEq)]
    pub enum MigrationState<S, C> {
        DynamicBlockSet,
        Stake(S),
        ConvertSubnets(C),
    }

    pub struct Migration<T: Config>(PhantomData<T>);

    impl<T: Config> SteppedMigration for Migration<T> {
        type Cursor = MigrationState<(T::AccountId, T::AccountId), u16>;
        type Identifier = MigrationId<16>;

        fn id() -> Self::Identifier {
            MigrationId { pallet_id: *PALLET_MIGRATIONS_ID, version_from: 6, version_to: 7 }
        }

        fn max_steps() -> Option<u32> {
            Some(10) // TODO: Make sure to change this to something that makes sense
        }

        fn step(
            mut cursor: Option<Self::Cursor>,
            meter: &mut WeightMeter,
        ) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
            if Pallet::<T>::on_chain_storage_version() != Self::id().version_from as u16 {
                return Ok(None);
            }

            let next = match cursor {
                None => Self::dynamic_block_step(),
                Some(MigrationState::DynamicBlockSet) => todo!(),
                _ => todo!(),
            };

            Ok(cursor)
        }
    }

    impl<T: Config> Migration<T> {
        fn dynamic_block_step() -> MigrationState<(T::AccountId, T::AccountId), u16> {
            DynamicBlock::<T>::set(Pallet::<T>::get_current_block_as_u64());
            MigrationState::DynamicBlockSet
        }
    }
}
