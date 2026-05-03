use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

#[subtensor_macros::freeze_struct("36db5a8053b1591e")]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug)]
pub struct ActiveValidatorStake<AuthorityId> {
    pub authority_id: AuthorityId,
    pub stake: u128,
    pub dkg_x25519_public_key: [u8; 32],
}

#[subtensor_macros::freeze_struct("35cdf9ee2db21525")]
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct DkgShareAtom<AuthorityId> {
    pub authority_id: AuthorityId,
    pub dkg_x25519_public_key: [u8; 32],
    pub share_id: u32,
    pub weight: u128,
}

#[subtensor_macros::freeze_struct("f2fdb604ecfdbf32")]
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct DkgAtomPlan<AuthorityId> {
    pub atoms: Vec<DkgShareAtom<AuthorityId>>,
    pub total_weight: u128,
    pub threshold_weight: u128,
}

#[derive(Clone, Eq, PartialEq, RuntimeDebug)]
pub enum DkgWeightingError {
    NoActiveValidators,
    TooManyValidatorsForAtomBudget,
    TooManyAtoms,
    ArithmeticOverflow,
}

pub fn two_thirds_plus_one(total_weight: u128) -> Result<u128, DkgWeightingError> {
    total_weight
        .checked_mul(2)
        .ok_or(DkgWeightingError::ArithmeticOverflow)
        .map(|x| x / 3 + 1)
}

pub fn plan_stake_weighted_atoms<AuthorityId: Clone + Ord>(
    validators: &[ActiveValidatorStake<AuthorityId>],
    max_atoms: u32,
) -> Result<DkgAtomPlan<AuthorityId>, DkgWeightingError> {
    let mut active: Vec<_> = validators.iter().filter(|v| v.stake > 0).cloned().collect();
    active.sort_by(|a, b| a.authority_id.cmp(&b.authority_id));

    if active.is_empty() {
        return Err(DkgWeightingError::NoActiveValidators);
    }
    if active.len() > max_atoms as usize {
        return Err(DkgWeightingError::TooManyValidatorsForAtomBudget);
    }

    let total_stake = active
        .iter()
        .try_fold(0u128, |acc, v| acc.checked_add(v.stake))
        .ok_or(DkgWeightingError::ArithmeticOverflow)?;

    let mut allocation: Vec<(AuthorityId, [u8; 32], u32, u128)> = Vec::with_capacity(active.len());
    let mut assigned = 0u32;

    for validator in &active {
        let scaled = validator
            .stake
            .checked_mul(max_atoms as u128)
            .ok_or(DkgWeightingError::ArithmeticOverflow)?;
        let base = (scaled / total_stake) as u32;
        let remainder = scaled % total_stake;
        let count = base.max(1);
        assigned = assigned
            .checked_add(count)
            .ok_or(DkgWeightingError::ArithmeticOverflow)?;
        allocation.push((
            validator.authority_id.clone(),
            validator.dkg_x25519_public_key,
            count,
            remainder,
        ));
    }

    while assigned > max_atoms {
        let Some(pos) = allocation
            .iter()
            .enumerate()
            .filter(|(_, (_, _, atoms, _))| *atoms > 1)
            .max_by_key(|(_, (_, _, atoms, _))| *atoms)
            .map(|(idx, _)| idx)
        else {
            return Err(DkgWeightingError::TooManyValidatorsForAtomBudget);
        };
        allocation[pos].2 -= 1;
        assigned -= 1;
    }

    while assigned < max_atoms {
        let Some(pos) = allocation
            .iter()
            .enumerate()
            .max_by_key(|(_, (_, _, _, rem))| *rem)
            .map(|(idx, _)| idx)
        else {
            break;
        };
        allocation[pos].2 = allocation[pos]
            .2
            .checked_add(1)
            .ok_or(DkgWeightingError::ArithmeticOverflow)?;
        assigned += 1;
    }

    let mut atoms = Vec::with_capacity(assigned as usize);
    let mut share_id = 1u32;
    for (authority_id, dkg_x25519_public_key, count, _) in allocation {
        for _ in 0..count {
            atoms.push(DkgShareAtom {
                authority_id: authority_id.clone(),
                dkg_x25519_public_key,
                share_id,
                weight: 1,
            });
            share_id = share_id
                .checked_add(1)
                .ok_or(DkgWeightingError::TooManyAtoms)?;
        }
    }

    let total_weight = atoms
        .iter()
        .fold(0u128, |acc, a| acc.saturating_add(a.weight));
    let threshold_weight = two_thirds_plus_one(total_weight)?;
    Ok(DkgAtomPlan {
        atoms,
        total_weight,
        threshold_weight,
    })
}

#[cfg(test)]
mod mev_shield_dkg_weighting_unit_tests {
    use super::*;
    use std::collections::BTreeMap;

    fn validator(id: u8, stake: u128) -> ActiveValidatorStake<Vec<u8>> {
        ActiveValidatorStake {
            authority_id: vec![id],
            stake,
            dkg_x25519_public_key: [id; 32],
        }
    }

    fn atom_counts(plan: &DkgAtomPlan<Vec<u8>>) -> BTreeMap<Vec<u8>, usize> {
        let mut counts = BTreeMap::new();
        for atom in &plan.atoms {
            *counts.entry(atom.authority_id.clone()).or_insert(0) += 1;
        }
        counts
    }

    #[test]
    fn threshold_is_floor_two_thirds_plus_one() {
        assert_eq!(two_thirds_plus_one(1).unwrap(), 1);
        assert_eq!(two_thirds_plus_one(2).unwrap(), 2);
        assert_eq!(two_thirds_plus_one(3).unwrap(), 3);
        assert_eq!(two_thirds_plus_one(4).unwrap(), 3);
        assert_eq!(two_thirds_plus_one(10).unwrap(), 7);
    }

    #[test]
    fn plan_rejects_empty_or_zero_stake_sets() {
        assert_eq!(
            plan_stake_weighted_atoms::<Vec<u8>>(&[], 8).unwrap_err(),
            DkgWeightingError::NoActiveValidators
        );
        assert_eq!(
            plan_stake_weighted_atoms(&[validator(1, 0), validator(2, 0)], 8).unwrap_err(),
            DkgWeightingError::NoActiveValidators
        );
    }

    #[test]
    fn plan_rejects_more_active_validators_than_atom_budget() {
        let validators = vec![validator(1, 1), validator(2, 1), validator(3, 1)];
        assert_eq!(
            plan_stake_weighted_atoms(&validators, 2).unwrap_err(),
            DkgWeightingError::TooManyValidatorsForAtomBudget
        );
    }

    #[test]
    fn plan_assigns_minimum_one_atom_and_consecutive_share_ids() {
        let plan =
            plan_stake_weighted_atoms(&[validator(1, 1), validator(2, 1_000), validator(3, 1)], 12)
                .unwrap();

        assert_eq!(plan.atoms.len(), 12);
        assert_eq!(plan.total_weight, 12);
        assert_eq!(plan.threshold_weight, 9);

        let counts = atom_counts(&plan);
        assert!(counts.get(&vec![1]).copied().unwrap_or_default() >= 1);
        assert!(counts.get(&vec![2]).copied().unwrap_or_default() >= 1);
        assert!(counts.get(&vec![3]).copied().unwrap_or_default() >= 1);

        let ids = plan.atoms.iter().map(|a| a.share_id).collect::<Vec<_>>();
        assert_eq!(ids, (1..=12).collect::<Vec<_>>());
        assert!(plan.atoms.iter().all(|a| a.weight == 1));
    }

    #[test]
    fn plan_ignores_zero_stake_validators_and_preserves_transport_keys() {
        let plan = plan_stake_weighted_atoms(&[validator(9, 0), validator(4, 10)], 4).unwrap();
        assert_eq!(plan.atoms.len(), 4);
        assert!(plan.atoms.iter().all(|a| a.authority_id == vec![4]));
        assert!(
            plan.atoms
                .iter()
                .all(|a| a.dkg_x25519_public_key == [4; 32])
        );
    }

    #[test]
    fn plan_is_deterministic_independent_of_input_order() {
        let a = vec![validator(3, 30), validator(1, 10), validator(2, 20)];
        let b = vec![validator(2, 20), validator(3, 30), validator(1, 10)];
        assert_eq!(
            plan_stake_weighted_atoms(&a, 12),
            plan_stake_weighted_atoms(&b, 12)
        );
    }

    #[test]
    fn plan_detects_stake_overflow() {
        assert_eq!(
            plan_stake_weighted_atoms(&[validator(1, u128::MAX), validator(2, u128::MAX)], 4)
                .unwrap_err(),
            DkgWeightingError::ArithmeticOverflow
        );
    }
}
