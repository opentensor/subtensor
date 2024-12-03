pragma solidity ^0.8.0;

address constant IMetagraph_ADDRESS = 0x0000000000000000000000000000000000000802;

struct AxonInfo {
  uint64 block;
  uint32 version;
  uint128 ip;
  uint16 port;
  uint8 ip_type;
  uint8 protocol;
}

interface IMetagraph {
  
  /**
   * @dev Returns the count of unique identifiers (UIDs) associated with a given network identifier (netuid).
   * @param netuid The network identifier for which to retrieve the UID count.
   * @return The count of UIDs associated with the specified netuid.
   */
  function getUidCount(uint16 netuid) external view returns (uint16);

  function getAxon(uint16 netuid, uint16 uid) external view returns (AxonInfo memory);

}
