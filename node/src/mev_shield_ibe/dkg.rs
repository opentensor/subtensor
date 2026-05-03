use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use codec::{Decode, Encode};
use parking_lot::RwLock;
use sp_core::H256;
use stp_mev_shield_ibe::{IbePendingIdentity, KEY_ID_LEN};

use super::crypto::{EpochDkgPublicOutput, EpochSecretShareBundle, PublicShareAtom};

#[subtensor_macros::freeze_struct("c8a8482c3e91353d")]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, Decode)]
pub struct DkgRoundKey {
    pub epoch: u64,
    pub key_id: [u8; KEY_ID_LEN],
}

#[subtensor_macros::freeze_struct("b0d21ec216f5e6cd")]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Encode, Decode)]
pub struct IdentityRoundKey {
    pub epoch: u64,
    pub target_block: u64,
    pub key_id: [u8; KEY_ID_LEN],
}

impl From<&IbePendingIdentity> for IdentityRoundKey {
    fn from(identity: &IbePendingIdentity) -> Self {
        Self {
            epoch: identity.epoch,
            target_block: identity.target_block,
            key_id: identity.key_id,
        }
    }
}

impl IdentityRoundKey {
    pub fn dkg_round_key(&self) -> DkgRoundKey {
        DkgRoundKey {
            epoch: self.epoch,
            key_id: self.key_id,
        }
    }
}

pub trait DkgKeySource: Send + Sync {
    fn genesis_hash(&self) -> H256;

    fn refresh(&self) -> Result<(), String>;

    fn secret_bundle_for_identity(
        &self,
        identity: &IdentityRoundKey,
    ) -> Option<Arc<EpochSecretShareBundle>>;

    fn public_output_for_identity(
        &self,
        identity: &IdentityRoundKey,
    ) -> Option<Arc<EpochDkgPublicOutput>>;

    fn public_atom_for_identity(
        &self,
        identity: &IdentityRoundKey,
        share_id: u32,
    ) -> Option<PublicShareAtom> {
        self.public_output_for_identity(identity)
            .and_then(|output| output.public_atom(share_id).cloned())
    }
}

/// The real DKG task writes completed epoch outputs through this trait.
///
/// The threshold-IBE share pool only reads from `DkgKeySource`; the DKG task writes
/// through `DkgOutputSink`. This keeps the share pool independent of the actual DKG
/// ceremony implementation.
pub trait DkgOutputSink: Send + Sync {
    fn upsert_public_output(&self, output: EpochDkgPublicOutput) -> Result<(), String>;
    fn upsert_secret_bundle(&self, bundle: EpochSecretShareBundle) -> Result<(), String>;
}

#[derive(Clone)]
pub struct ProductionDkgKeySource {
    inner: Arc<Inner>,
}

struct Inner {
    genesis_hash: H256,
    root_dir: PathBuf,

    public_outputs: RwLock<BTreeMap<DkgRoundKey, Arc<EpochDkgPublicOutput>>>,
    secret_bundles: RwLock<BTreeMap<DkgRoundKey, Arc<EpochSecretShareBundle>>>,
}

impl ProductionDkgKeySource {
    pub fn new(genesis_hash: H256, root_dir: impl Into<PathBuf>) -> Result<Self, String> {
        let source = Self {
            inner: Arc::new(Inner {
                genesis_hash,
                root_dir: root_dir.into(),
                public_outputs: RwLock::new(BTreeMap::new()),
                secret_bundles: RwLock::new(BTreeMap::new()),
            }),
        };

        source.ensure_dirs()?;
        source.refresh()?;

        Ok(source)
    }

    pub fn public_dir(&self) -> PathBuf {
        self.inner.root_dir.join("public")
    }

    pub fn secret_dir(&self) -> PathBuf {
        self.inner.root_dir.join("secret")
    }

    fn ensure_dirs(&self) -> Result<(), String> {
        fs::create_dir_all(self.public_dir()).map_err(|e| format!("create public DKG dir: {e}"))?;

        fs::create_dir_all(self.secret_dir()).map_err(|e| format!("create secret DKG dir: {e}"))?;

        Ok(())
    }

    fn file_name(key: &DkgRoundKey) -> String {
        format!("epoch-{}-{}.scale", key.epoch, hex::encode(key.key_id))
    }

    fn public_path(&self, key: &DkgRoundKey) -> PathBuf {
        self.public_dir().join(Self::file_name(key))
    }

    fn secret_path(&self, key: &DkgRoundKey) -> PathBuf {
        self.secret_dir().join(Self::file_name(key))
    }

    fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
        let parent = path
            .parent()
            .ok_or_else(|| format!("path has no parent: {}", path.display()))?;

        fs::create_dir_all(parent)
            .map_err(|e| format!("create parent dir {}: {e}", parent.display()))?;

        let tmp = path.with_extension(format!("tmp-{}", std::process::id()));

        {
            let mut file = fs::File::create(&tmp)
                .map_err(|e| format!("create temp file {}: {e}", tmp.display()))?;

            file.write_all(bytes)
                .map_err(|e| format!("write temp file {}: {e}", tmp.display()))?;

            file.sync_all()
                .map_err(|e| format!("sync temp file {}: {e}", tmp.display()))?;
        }

        fs::rename(&tmp, path).map_err(|e| {
            format!(
                "rename temp file {} to {}: {e}",
                tmp.display(),
                path.display()
            )
        })?;

        Ok(())
    }

    fn read_scale_file<T: Decode>(path: &Path) -> Result<T, String> {
        let bytes = fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;

        let mut input = &bytes[..];

        let value = T::decode(&mut input)
            .map_err(|e| format!("decode SCALE file {}: {e:?}", path.display()))?;

        if !input.is_empty() {
            return Err(format!(
                "trailing bytes in SCALE file {}: {} bytes",
                path.display(),
                input.len()
            ));
        }

        Ok(value)
    }

    fn round_key_from_public(output: &EpochDkgPublicOutput) -> DkgRoundKey {
        DkgRoundKey {
            epoch: output.epoch_key.epoch,
            key_id: output.epoch_key.key_id,
        }
    }

    fn round_key_from_secret(bundle: &EpochSecretShareBundle) -> DkgRoundKey {
        Self::round_key_from_public(&bundle.public)
    }

    fn validate_public_output(output: &EpochDkgPublicOutput) -> Result<(), String> {
        let epoch_key = &output.epoch_key;

        if output.public_atoms.is_empty() {
            return Err("public DKG output has no atoms".into());
        }

        if epoch_key.total_weight == 0 {
            return Err("epoch total_weight is zero".into());
        }

        if epoch_key.threshold_weight == 0 {
            return Err("epoch threshold_weight is zero".into());
        }

        if epoch_key.threshold_weight > epoch_key.total_weight {
            return Err(format!(
                "threshold_weight {} exceeds total_weight {}",
                epoch_key.threshold_weight, epoch_key.total_weight
            ));
        }

        let atom_weight = output.total_public_weight();

        if atom_weight != epoch_key.total_weight {
            return Err(format!(
                "public atom weight sum {atom_weight} != epoch total_weight {}",
                epoch_key.total_weight
            ));
        }

        let mut ids = BTreeSet::new();

        for atom in &output.public_atoms {
            if atom.share_id == 0 {
                return Err("share_id 0 is invalid for Shamir interpolation".into());
            }

            if atom.weight == 0 {
                return Err(format!("share_id {} has zero weight", atom.share_id));
            }

            if atom.public_share.is_empty() {
                return Err(format!("share_id {} has empty public_share", atom.share_id));
            }

            if !ids.insert(atom.share_id) {
                return Err(format!("duplicate share_id {}", atom.share_id));
            }
        }

        Ok(())
    }

    fn validate_secret_bundle(bundle: &EpochSecretShareBundle) -> Result<(), String> {
        Self::validate_public_output(&bundle.public)?;

        if bundle.local_atoms.is_empty() {
            return Err("secret bundle has no local atoms".into());
        }

        for atom in &bundle.local_atoms {
            if atom.secret_scalar.is_empty() {
                return Err(format!(
                    "local share_id {} has empty secret_scalar",
                    atom.public.share_id
                ));
            }

            let Some(public_atom) = bundle.public.public_atom(atom.public.share_id) else {
                return Err(format!(
                    "local share_id {} is missing from public output",
                    atom.public.share_id
                ));
            };

            if public_atom.weight != atom.public.weight {
                return Err(format!(
                    "local share_id {} weight mismatch",
                    atom.public.share_id
                ));
            }

            if public_atom.public_share != atom.public.public_share {
                return Err(format!(
                    "local share_id {} public_share mismatch",
                    atom.public.share_id
                ));
            }
        }

        Ok(())
    }

    fn load_public_outputs(
        &self,
    ) -> Result<BTreeMap<DkgRoundKey, Arc<EpochDkgPublicOutput>>, String> {
        let mut outputs = BTreeMap::new();

        if !self.public_dir().exists() {
            return Ok(outputs);
        }

        for entry in
            fs::read_dir(self.public_dir()).map_err(|e| format!("read public DKG dir: {e}"))?
        {
            let entry = entry.map_err(|e| format!("read public DKG dir entry: {e}"))?;
            let path = entry.path();

            if path.extension().and_then(|x| x.to_str()) != Some("scale") {
                continue;
            }

            let output: EpochDkgPublicOutput = Self::read_scale_file(&path)?;
            Self::validate_public_output(&output)?;

            outputs.insert(Self::round_key_from_public(&output), Arc::new(output));
        }

        Ok(outputs)
    }

    fn load_secret_bundles(
        &self,
    ) -> Result<BTreeMap<DkgRoundKey, Arc<EpochSecretShareBundle>>, String> {
        let mut bundles = BTreeMap::new();

        if !self.secret_dir().exists() {
            return Ok(bundles);
        }

        for entry in
            fs::read_dir(self.secret_dir()).map_err(|e| format!("read secret DKG dir: {e}"))?
        {
            let entry = entry.map_err(|e| format!("read secret DKG dir entry: {e}"))?;
            let path = entry.path();

            if path.extension().and_then(|x| x.to_str()) != Some("scale") {
                continue;
            }

            let bundle: EpochSecretShareBundle = Self::read_scale_file(&path)?;
            Self::validate_secret_bundle(&bundle)?;

            bundles.insert(Self::round_key_from_secret(&bundle), Arc::new(bundle));
        }

        Ok(bundles)
    }
}

impl DkgKeySource for ProductionDkgKeySource {
    fn genesis_hash(&self) -> H256 {
        self.inner.genesis_hash
    }

    fn refresh(&self) -> Result<(), String> {
        self.ensure_dirs()?;

        let public_outputs = self.load_public_outputs()?;
        let secret_bundles = self.load_secret_bundles()?;

        *self.inner.public_outputs.write() = public_outputs;
        *self.inner.secret_bundles.write() = secret_bundles;

        Ok(())
    }

    fn secret_bundle_for_identity(
        &self,
        identity: &IdentityRoundKey,
    ) -> Option<Arc<EpochSecretShareBundle>> {
        self.inner
            .secret_bundles
            .read()
            .get(&identity.dkg_round_key())
            .cloned()
    }

    fn public_output_for_identity(
        &self,
        identity: &IdentityRoundKey,
    ) -> Option<Arc<EpochDkgPublicOutput>> {
        let key = identity.dkg_round_key();

        if let Some(output) = self.inner.public_outputs.read().get(&key).cloned() {
            return Some(output);
        }

        self.inner
            .secret_bundles
            .read()
            .get(&key)
            .map(|bundle| Arc::new(bundle.public.clone()))
    }
}

impl DkgOutputSink for ProductionDkgKeySource {
    fn upsert_public_output(&self, output: EpochDkgPublicOutput) -> Result<(), String> {
        Self::validate_public_output(&output)?;

        let key = Self::round_key_from_public(&output);
        let path = self.public_path(&key);

        Self::atomic_write(&path, &output.encode())?;

        self.inner
            .public_outputs
            .write()
            .insert(key, Arc::new(output));

        Ok(())
    }

    fn upsert_secret_bundle(&self, bundle: EpochSecretShareBundle) -> Result<(), String> {
        Self::validate_secret_bundle(&bundle)?;

        let key = Self::round_key_from_secret(&bundle);

        let public_path = self.public_path(&key);
        let secret_path = self.secret_path(&key);

        Self::atomic_write(&public_path, &bundle.public.encode())?;
        Self::atomic_write(&secret_path, &bundle.encode())?;

        self.inner
            .public_outputs
            .write()
            .insert(key.clone(), Arc::new(bundle.public.clone()));

        self.inner
            .secret_bundles
            .write()
            .insert(key, Arc::new(bundle));

        Ok(())
    }
}
