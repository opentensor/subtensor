extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

use fp_evm::PrecompileHandle;
use frame_support::__private::metadata_ir::{
    MetadataIR, StorageEntryModifierIR, StorageEntryTypeIR, StorageHasherIR,
};
use precompile_utils::prelude::{UnboundedBytes, revert};
use precompile_utils::EvmResult;
use sp_core::ByteArray;

use crate::PrecompileExt;

/// Trait that the runtime must implement to provide metadata IR.
pub trait RuntimeMetadataProvider {
    fn metadata_ir() -> MetadataIR;
}

pub struct GetStoragePrecompile<R>(PhantomData<R>);

impl<R> PrecompileExt<R::AccountId> for GetStoragePrecompile<R>
where
    R: frame_system::Config + pallet_subtensor::Config + RuntimeMetadataProvider,
    R::AccountId: From<[u8; 32]> + ByteArray,
{
    const INDEX: u64 = 2062;
}

/// A filtered pallet storage entry with its metadata.
struct PalletStorageInfo {
    pallet_name: &'static str,
    entries: Vec<EntryInfo>,
}

struct EntryInfo {
    name: &'static str,
    storage_type: u8,
    modifier: u8,
    hashers: String,
    key_type: String,
    value_type: String,
}

/// Build a list of pallets that have storage, along with their entries.
fn build_storage_info<R: RuntimeMetadataProvider>() -> Vec<PalletStorageInfo> {
    let metadata = R::metadata_ir();
    let mut result = Vec::new();

    for pallet in metadata.pallets {
        if let Some(storage) = pallet.storage {
            let mut entries = Vec::new();
            for entry in storage.entries {
                let (storage_type, hashers_str, key_type_str) = match &entry.ty {
                    StorageEntryTypeIR::Plain(_) => {
                        (0u8, String::new(), String::new())
                    }
                    StorageEntryTypeIR::Map {
                        hashers,
                        key,
                        ..
                    } => {
                        let hashers_str = hashers
                            .iter()
                            .map(hasher_to_str)
                            .collect::<Vec<_>>()
                            .join(",");
                        let key_type_str = meta_type_to_string(key);
                        (1u8, hashers_str, key_type_str)
                    }
                };

                let value_type_str = match &entry.ty {
                    StorageEntryTypeIR::Plain(v) => meta_type_to_string(v),
                    StorageEntryTypeIR::Map { value, .. } => meta_type_to_string(value),
                };

                let modifier = match entry.modifier {
                    StorageEntryModifierIR::Optional => 0u8,
                    StorageEntryModifierIR::Default => 1u8,
                };

                entries.push(EntryInfo {
                    name: entry.name,
                    storage_type,
                    modifier,
                    hashers: hashers_str,
                    key_type: key_type_str,
                    value_type: value_type_str,
                });
            }

            result.push(PalletStorageInfo {
                pallet_name: pallet.name,
                entries,
            });
        }
    }

    result
}

fn hasher_to_str(h: &StorageHasherIR) -> &'static str {
    match h {
        StorageHasherIR::Blake2_128 => "Blake2_128",
        StorageHasherIR::Blake2_256 => "Blake2_256",
        StorageHasherIR::Blake2_128Concat => "Blake2_128Concat",
        StorageHasherIR::Twox128 => "Twox128",
        StorageHasherIR::Twox256 => "Twox256",
        StorageHasherIR::Twox64Concat => "Twox64Concat",
        StorageHasherIR::Identity => "Identity",
    }
}

fn meta_type_to_string(meta: &scale_info::MetaType) -> String {
    let ty = meta.type_info();
    let segments = &ty.path.segments;
    if segments.is_empty() {
        // For primitive or unnamed types, use the type_def to produce a name
        format_type_def(&ty.type_def)
    } else {
        // Use the last segment as the readable type name
        segments
            .last()
            .map(|s| String::from(*s))
            .unwrap_or_default()
    }
}

fn format_type_def(type_def: &scale_info::TypeDef) -> String {
    match type_def {
        scale_info::TypeDef::Composite(composite) => {
            let fields: Vec<String> = composite
                .fields
                .iter()
                .map(|f| {
                    f.name
                        .map(String::from)
                        .unwrap_or_else(|| meta_type_to_string(&f.ty))
                })
                .collect();
            if fields.is_empty() {
                String::from("()")
            } else {
                let mut out = String::from("(");
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(field);
                }
                out.push(')');
                out
            }
        }
        scale_info::TypeDef::Tuple(tuple) => {
            let fields: Vec<String> = tuple.fields.iter().map(meta_type_to_string).collect();
            let mut out = String::from("(");
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(field);
            }
            out.push(')');
            out
        }
        scale_info::TypeDef::Sequence(seq) => {
            let inner = meta_type_to_string(&seq.type_param);
            let mut out = String::from("Vec<");
            out.push_str(&inner);
            out.push('>');
            out
        }
        scale_info::TypeDef::Array(arr) => {
            let inner = meta_type_to_string(&arr.type_param);
            let mut out = String::from("[");
            out.push_str(&inner);
            out.push_str("; ");
            let mut len_str = String::new();
            format_u32_to_string(arr.len, &mut len_str);
            out.push_str(&len_str);
            out.push(']');
            out
        }
        scale_info::TypeDef::Compact(compact) => {
            let inner = meta_type_to_string(&compact.type_param);
            let mut out = String::from("Compact<");
            out.push_str(&inner);
            out.push('>');
            out
        }
        scale_info::TypeDef::Primitive(prim) => String::from(match prim {
            scale_info::TypeDefPrimitive::Bool => "bool",
            scale_info::TypeDefPrimitive::Char => "char",
            scale_info::TypeDefPrimitive::Str => "str",
            scale_info::TypeDefPrimitive::U8 => "u8",
            scale_info::TypeDefPrimitive::U16 => "u16",
            scale_info::TypeDefPrimitive::U32 => "u32",
            scale_info::TypeDefPrimitive::U64 => "u64",
            scale_info::TypeDefPrimitive::U128 => "u128",
            scale_info::TypeDefPrimitive::U256 => "u256",
            scale_info::TypeDefPrimitive::I8 => "i8",
            scale_info::TypeDefPrimitive::I16 => "i16",
            scale_info::TypeDefPrimitive::I32 => "i32",
            scale_info::TypeDefPrimitive::I64 => "i64",
            scale_info::TypeDefPrimitive::I128 => "i128",
            scale_info::TypeDefPrimitive::I256 => "i256",
        }),
        scale_info::TypeDef::Variant(_) => String::from("Enum"),
        scale_info::TypeDef::BitSequence(_) => String::from("BitSequence"),
    }
}

/// Format a u32 to string without using arithmetic operators.
fn format_u32_to_string(val: u32, out: &mut String) {
    if val == 0 {
        out.push('0');
        return;
    }
    let mut digits = Vec::new();
    let mut remaining = val;
    while remaining > 0 {
        let (div, rem) = (remaining.saturating_div(10), remaining.checked_rem(10).unwrap_or(0));
        // Convert digit to char
        let c = match rem {
            0 => '0',
            1 => '1',
            2 => '2',
            3 => '3',
            4 => '4',
            5 => '5',
            6 => '6',
            7 => '7',
            8 => '8',
            9 => '9',
            _ => '?',
        };
        digits.push(c);
        remaining = div;
    }
    for d in digits.iter().rev() {
        out.push(*d);
    }
}

#[precompile_utils::precompile]
impl<R> GetStoragePrecompile<R>
where
    R: frame_system::Config + pallet_subtensor::Config + RuntimeMetadataProvider,
    R::AccountId: ByteArray,
{
    /// Returns the number of pallets that have storage entries.
    #[precompile::public("getPalletCount()")]
    #[precompile::view]
    fn get_pallet_count(_: &mut impl PrecompileHandle) -> EvmResult<u32> {
        let info = build_storage_info::<R>();
        Ok(info.len() as u32)
    }

    /// Returns the name of the pallet at the given index (among pallets with storage).
    #[precompile::public("getPalletName(uint32)")]
    #[precompile::view]
    fn get_pallet_name(
        _: &mut impl PrecompileHandle,
        pallet_index: u32,
    ) -> EvmResult<UnboundedBytes> {
        let info = build_storage_info::<R>();
        let pallet = info.get(pallet_index as usize).ok_or(revert(
            "pallet index out of bounds",
        ))?;
        Ok(pallet.pallet_name.as_bytes().to_vec().into())
    }

    /// Returns the number of storage entries for the pallet at the given index.
    #[precompile::public("getEntryCount(uint32)")]
    #[precompile::view]
    fn get_entry_count(
        _: &mut impl PrecompileHandle,
        pallet_index: u32,
    ) -> EvmResult<u32> {
        let info = build_storage_info::<R>();
        let pallet = info.get(pallet_index as usize).ok_or(revert(
            "pallet index out of bounds",
        ))?;
        Ok(pallet.entries.len() as u32)
    }

    /// Returns details for a specific storage entry.
    ///
    /// Returns (storageName, storageType, modifier, hashers, keyType, valueType) where:
    /// - storageType: 0=Plain (StorageValue), 1=Map (StorageMap/DoubleMap/NMap)
    /// - modifier: 0=Optional, 1=Default
    /// - hashers: comma-separated hasher names
    /// - keyType/valueType: human-readable type path
    #[precompile::public("getEntryDetails(uint32,uint32)")]
    #[precompile::view]
    fn get_entry_details(
        _: &mut impl PrecompileHandle,
        pallet_index: u32,
        entry_index: u32,
    ) -> EvmResult<(UnboundedBytes, u8, u8, UnboundedBytes, UnboundedBytes, UnboundedBytes)> {
        let info = build_storage_info::<R>();
        let pallet = info.get(pallet_index as usize).ok_or(revert(
            "pallet index out of bounds",
        ))?;
        let entry = pallet
            .entries
            .get(entry_index as usize)
            .ok_or(revert("entry index out of bounds"))?;

        Ok((
            entry.name.as_bytes().to_vec().into(),
            entry.storage_type,
            entry.modifier,
            entry.hashers.as_bytes().to_vec().into(),
            entry.key_type.as_bytes().to_vec().into(),
            entry.value_type.as_bytes().to_vec().into(),
        ))
    }
}
