pragma solidity ^0.8.0;

address constant IMetagraph_ADDRESS = 0x0000000000000000000000000000000000000802;

interface IMetagraph {
  
  /**
   * @dev Returns the count of unique identifiers (UIDs) associated with a given network identifier (netuid).
   * @param netuid The network identifier for which to retrieve the UID count.
   * @return The count of UIDs associated with the specified netuid.
   */
  function getUidCount(uint16 netuid) external view returns (uint16);

}
