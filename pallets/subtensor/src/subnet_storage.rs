use sp_runtime::paste;
use frame_support::traits::StorageInstance;



GenerateStorageValue!(              u64,            SenateRequiredStakePercentage,          T::InitialSenateRequiredStakePercentage::get());

GenerateStorageValue!(              u64,            DefaultBlockEmission,                   0u64);
GenerateStorageValue!(              u64,            TotalStake,                             0u64);
GenerateStorageValue!(              u16,            DefaultTake,                            T::InitialDefaultTake::get());
GenerateStorageMap!(u16,            u16,            Delegates,                              T::InitialDefaultTake::get());
