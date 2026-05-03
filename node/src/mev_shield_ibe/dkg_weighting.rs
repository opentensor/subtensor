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
