### netuid 3
```rust
Rho: u16 = 10;
Kappa: u16 = 32_767; // 0.5 = 65535/2 
MaxAllowedUids: u16 = 512;
Issuance: u64 = 0;
MinAllowedWeights: u16 = 1024;
EmissionValue: u16 = 0;
MaxWeightsLimit: u16 = 262;
ValidatorBatchSize: u16 = 10; // 32
ValidatorSequenceLen: u16 = 10; // 256
ValidatorEpochLen: u16 = 100;
ValidatorEpochsPerReset: u16 = 60;
ValidatorExcludeQuantile: u16 = 6554; // 10% of u16
ValidatorPruneLen: u64 = 1;
ValidatorLogitsDivergence: u16 = 1310; // 2% of u16
ScalingLawPower: u16 = 50; // 0.5
SynergyScalingLawPower: u16 = 50; // 0.5
MaxAllowedValidators: u16 = 100;
Tempo: u16 = 0;
Difficulty: u64 = 1;
AdjustmentInterval: u16 = 100;
TargetRegistrationsPerInterval: u16 = 2;
ImmunityPeriod: u16 = 4096;
ActivityCutoff: u16 = 5000;
MaxRegistrationsPerBlock: u16 = 50;
PruningScore : u16 = u16::MAX;
BondsMovingAverage: u64 = 900_000;
DefaultTake: u16 = 11_796; // 18% honest number.
WeightsVersionKey: u64 = 370;
MinDifficulty: u64 = 1;
MaxDifficulty: u64 = 10;
ServingRateLimit: u64 = 1000; 
Burn: u64 = 0; 
MinBurn: u64 = 0; 
MaxBurn: u64 = 1_000_000_000;
TxRateLimit: u64 = 1000;
```