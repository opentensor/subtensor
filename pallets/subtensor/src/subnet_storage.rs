use sp_runtime::paste;
use frame_support::traits::StorageInstance;

macro_rules! GenerateStorageValue
{
    ($a:ty, $b:ident, $c:stmt) =>
    {
        paste::paste!
        {
            pub struct [<Default $b>]
            {
            }

            impl [<Default $b>] where 
            StorageInstance: frame_support::storage::StorageInstance
            {
                pub fn Get<T: Config>() -> $a
                {
                    $c
                }
            }

            pub type $b<T> = StorageValue<T, $a, ValueQuery, [<Default $b>]>;
        }
    }
}

macro_rules! GenerateStorageMap
{
    ($a:ty, $b:ty, $c:ident, $d:stmt) =>
    {
        paste::paste!
        {
            pub struct [<Default $c>]
            {
            }

            impl [<Default $c>] where 
            StorageInstance: frame_support::traits::StorageInstance
            {
                pub fn Get<T: Config>() -> $a
                {
                    $d
                }
            }

            pub type $c<T> = StorageMap<T, Blake2_128Concat, $a, ValueQuery, [<Default $c>]>;
        }
    }
}

GenerateStorageValue!(              u64,            SenateRequiredStakePercentage,          T::InitialSenateRequiredStakePercentage::get());

GenerateStorageValue!(              u64,            DefaultBlockEmission,                   0u64);
GenerateStorageValue!(              u64,            TotalStake,                             0u64);
GenerateStorageValue!(              u16,            DefaultTake,                            T::InitialDefaultTake::get());
GenerateStorageMap!(u16,            u16,            Delegates,                              T::InitialDefaultTake::get());
