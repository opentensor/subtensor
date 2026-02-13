// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

address constant IGETSTORAGE_ADDRESS = 0x000000000000000000000000000000000000080E; // 2062

interface IGetStorage {
    /// @notice Returns the number of pallets that have storage entries.
    function getPalletCount() external view returns (uint32);

    /// @notice Returns the name of the pallet at the given index.
    /// @param palletIndex Index of the pallet (0-based, among pallets with storage).
    function getPalletName(uint32 palletIndex) external view returns (bytes memory);

    /// @notice Returns the number of storage entries for the pallet at the given index.
    /// @param palletIndex Index of the pallet.
    function getEntryCount(uint32 palletIndex) external view returns (uint32);

    /// @notice Returns details for a specific storage entry.
    /// @param palletIndex Index of the pallet.
    /// @param entryIndex Index of the storage entry within the pallet.
    /// @return storageName Name of the storage entry.
    /// @return storageType 0=Plain (StorageValue), 1=Map (StorageMap/DoubleMap/NMap).
    /// @return modifier 0=Optional, 1=Default.
    /// @return hashers Comma-separated hasher names (e.g. "Blake2_128Concat" or "Identity,Blake2_128Concat").
    /// @return keyType Human-readable type path for the key.
    /// @return valueType Human-readable type path for the value.
    function getEntryDetails(uint32 palletIndex, uint32 entryIndex) external view returns (
        bytes memory storageName,
        uint8 storageType,
        uint8 modifier,
        bytes memory hashers,
        bytes memory keyType,
        bytes memory valueType
    );
}
