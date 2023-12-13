use sp_runtime::paste;

macro_rules! GenerateStorageValue
{
    ($a:ty, $b:ident, $c:stmt) =>
    {
        paste::paste!
        {
            pub struct [<Default $b>]
            {
            }

            impl [<Default $b>]
            {
                fn Get<T: Config>() -> $a
                {
                    $c
                }
            }

            type $b<T> = StorageValue<T, $a, ValueQuery, [<Default $b>]>;
        }
    }
}

GenerateStorageValue!(u64, SenateRequiredStakePercentage, T::InitialSenateRequiredStakePercentage::get());