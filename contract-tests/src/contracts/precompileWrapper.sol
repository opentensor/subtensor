// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Precompile addresses
address constant ISUBTENSOR_BALANCE_TRANSFER_ADDRESS = 0x0000000000000000000000000000000000000800;
address constant IMETAGRAPH_ADDRESS = 0x0000000000000000000000000000000000000802;
address constant ISUBNET_ADDRESS = 0x0000000000000000000000000000000000000803;
address constant INEURON_ADDRESS = 0x0000000000000000000000000000000000000804;
address constant ISTAKING_V2_ADDRESS = 0x0000000000000000000000000000000000000805;
address constant IUID_LOOKUP_ADDRESS = 0x0000000000000000000000000000000000000806;
address constant IALPHA_ADDRESS = 0x0000000000000000000000000000000000000808;
address constant ICROWDLOAN_ADDRESS = 0x0000000000000000000000000000000000000809;
address constant ILEASING_ADDRESS = 0x000000000000000000000000000000000000080a;
address constant IPROXY_ADDRESS = 0x000000000000000000000000000000000000080b;
address constant IADDRESS_MAPPING_ADDRESS = 0x000000000000000000000000000000000000080C;

// Interface definitions
interface ISubtensorBalanceTransfer {
    function transfer(bytes32 data) external payable;
}

interface IMetagraph {
    function getUidCount(uint16 netuid) external view returns (uint16);
}

interface ISubnet {
    function registerNetwork(
        bytes32 hotkey,
        string memory subnetName,
        string memory githubRepo,
        string memory subnetContact,
        string memory subnetUrl,
        string memory discord,
        string memory description,
        string memory additional
    ) external payable;
    function getServingRateLimit(uint16 netuid) external view returns (uint64);
}

interface INeuron {
    function burnedRegister(uint16 netuid, bytes32 hotkey) external payable;
}

interface IStaking {
    function addStake(
        bytes32 hotkey,
        uint256 amount,
        uint256 netuid
    ) external payable;
    function removeStake(
        bytes32 hotkey,
        uint256 amount,
        uint256 netuid
    ) external payable;
    function getTotalColdkeyStake(
        bytes32 coldkey
    ) external view returns (uint256);
    function getTotalHotkeyStake(
        bytes32 hotkey
    ) external view returns (uint256);
}

struct LookupItem {
    uint16 uid;
    uint64 block_associated;
}

interface IUidLookup {
    function uidLookup(
        uint16 netuid,
        address evm_address,
        uint16 limit
    ) external view returns (LookupItem[] memory);
}

interface IAlpha {
    function getAlphaPrice(uint16 netuid) external view returns (uint256);
}

struct CrowdloanInfo {
    bytes32 creator;
    uint64 deposit;
    uint64 min_contribution;
    uint32 end;
    uint64 cap;
    bytes32 funds_account;
    uint64 raised;
    bool has_target_address;
    bytes32 target_address;
    bool finalized;
    uint32 contributors_count;
}

interface ICrowdloan {
    function getCrowdloan(
        uint32 crowdloanId
    ) external view returns (CrowdloanInfo memory);
    function getContribution(
        uint32 crowdloanId,
        bytes32 coldkey
    ) external view returns (uint64);
    function create(
        uint64 deposit,
        uint64 minContribution,
        uint64 cap,
        uint32 end,
        address targetAddress
    ) external payable;
}

struct LeaseInfo {
    bytes32 beneficiary;
    bytes32 coldkey;
    bytes32 hotkey;
    uint8 emissions_share;
    bool has_end_block;
    uint32 end_block;
    uint16 netuid;
    uint64 cost;
}

interface ILeasing {
    function getContributorShare(
        uint32 leaseId,
        bytes32 contributor
    ) external view returns (uint128, uint128);
    function createLeaseCrowdloan(
        uint64 crowdloanDeposit,
        uint64 crowdloanMinContribution,
        uint64 crowdloanCap,
        uint32 crowdloanEnd,
        uint8 leasingEmissionsShare,
        bool hasLeasingEndBlock,
        uint32 leasingEndBlock
    ) external payable;
}

interface IProxy {
    struct ProxyInfo {
        bytes32 delegate;
        uint256 proxy_type;
        uint256 delay;
    }

    function addProxy(
        bytes32 delegate,
        uint8 proxy_type,
        uint32 delay
    ) external;
    function proxyCall(
        bytes32 real,
        uint8[] memory force_proxy_type,
        uint8[] memory call
    ) external;
    function getProxies(
        bytes32 account
    ) external view returns (ProxyInfo[] memory);
}

interface IAddressMapping {
    function addressMapping(
        address target_address
    ) external view returns (bytes32);
}

/**
 * @title PrecompileWrapper
 * @dev A wrapper contract that calls all precompile functions directly
 * instead of using low-level calls like address.call()
 */
contract PrecompileWrapper {
    ISubtensorBalanceTransfer public constant balanceTransfer =
        ISubtensorBalanceTransfer(ISUBTENSOR_BALANCE_TRANSFER_ADDRESS);
    IMetagraph public constant metagraph = IMetagraph(IMETAGRAPH_ADDRESS);
    ISubnet public constant subnet = ISubnet(ISUBNET_ADDRESS);
    INeuron public constant neuron = INeuron(INEURON_ADDRESS);
    IStaking public constant staking = IStaking(ISTAKING_V2_ADDRESS);
    IUidLookup public constant uidLookupPrecompile =
        IUidLookup(IUID_LOOKUP_ADDRESS);
    IAlpha public constant alpha = IAlpha(IALPHA_ADDRESS);
    ICrowdloan public constant crowdloan = ICrowdloan(ICROWDLOAN_ADDRESS);
    ILeasing public constant leasing = ILeasing(ILEASING_ADDRESS);
    IProxy public constant proxy = IProxy(IPROXY_ADDRESS);
    IAddressMapping public constant addressMappingPrecompile =
        IAddressMapping(IADDRESS_MAPPING_ADDRESS);

    // ============ SubtensorBalanceTransfer Functions ============
    function transfer(bytes32 data) external payable {
        balanceTransfer.transfer{value: msg.value}(data);
    }

    // ============ Metagraph Functions ============

    function getUidCount(uint16 netuid) external view returns (uint16) {
        return metagraph.getUidCount(netuid);
    }

    // ============ Subnet Functions ============

    function registerNetworkWithDetails(
        bytes32 hotkey,
        string memory subnetName,
        string memory githubRepo,
        string memory subnetContact,
        string memory subnetUrl,
        string memory discord,
        string memory description,
        string memory additional
    ) external payable {
        subnet.registerNetwork(
            hotkey,
            subnetName,
            githubRepo,
            subnetContact,
            subnetUrl,
            discord,
            description,
            additional
        );
    }

    function getServingRateLimit(uint16 netuid) external view returns (uint64) {
        return subnet.getServingRateLimit(netuid);
    }

    // ============ Neuron Functions ============

    function burnedRegister(uint16 netuid, bytes32 hotkey) external payable {
        neuron.burnedRegister{value: msg.value}(netuid, hotkey);
    }

    // ============ Staking Functions ============
    function addStake(
        bytes32 hotkey,
        uint256 amount,
        uint256 netuid
    ) external payable {
        staking.addStake(hotkey, amount, netuid);
    }

    function removeStake(
        bytes32 hotkey,
        uint256 amount,
        uint256 netuid
    ) external payable {
        staking.removeStake(hotkey, amount, netuid);
    }

    function getTotalColdkeyStake(
        bytes32 coldkey
    ) external view returns (uint256) {
        return staking.getTotalColdkeyStake(coldkey);
    }

    function getTotalHotkeyStake(
        bytes32 hotkey
    ) external view returns (uint256) {
        return staking.getTotalHotkeyStake(hotkey);
    }

    // ============ Alpha Functions ============

    function getAlphaPrice(uint16 netuid) external view returns (uint256) {
        return alpha.getAlphaPrice(netuid);
    }

    // ============ Address Mapping Functions ============

    function addressMapping(
        address target_address
    ) external view returns (bytes32) {
        return addressMappingPrecompile.addressMapping(target_address);
    }

    // ============ Proxy Functions ============

    function proxyCall(
        bytes32 real,
        uint8[] memory force_proxy_type,
        uint8[] memory call
    ) external {
        proxy.proxyCall(real, force_proxy_type, call);
    }

    function addProxy(
        bytes32 delegate,
        uint8 proxy_type,
        uint32 delay
    ) external {
        proxy.addProxy(delegate, proxy_type, delay);
    }

    function getProxies(
        bytes32 account
    ) external view returns (IProxy.ProxyInfo[] memory) {
        return proxy.getProxies(account);
    }

    // ============ UID Lookup Functions ============

    function uidLookup(
        uint16 netuid,
        address evm_address,
        uint16 limit
    ) external view returns (LookupItem[] memory) {
        return uidLookupPrecompile.uidLookup(netuid, evm_address, limit);
    }

    // ============ Crowdloan Functions ============

    function getCrowdloan(
        uint32 crowdloanId
    ) external view returns (CrowdloanInfo memory) {
        return crowdloan.getCrowdloan(crowdloanId);
    }

    function getContribution(
        uint32 crowdloanId,
        bytes32 coldkey
    ) external view returns (uint64) {
        return crowdloan.getContribution(crowdloanId, coldkey);
    }

    function createCrowdloan(
        uint64 deposit,
        uint64 minContribution,
        uint64 cap,
        uint32 end,
        address targetAddress
    ) external payable {
        crowdloan.create(deposit, minContribution, cap, end, targetAddress);
    }

    // ============ Leasing Functions ============

    function getContributorShare(
        uint32 leaseId,
        bytes32 contributor
    ) external view returns (uint128, uint128) {
        return leasing.getContributorShare(leaseId, contributor);
    }

    function createLeaseCrowdloan(
        uint64 crowdloanDeposit,
        uint64 crowdloanMinContribution,
        uint64 crowdloanCap,
        uint32 crowdloanEnd,
        uint8 leasingEmissionsShare,
        bool hasLeasingEndBlock,
        uint32 leasingEndBlock
    ) external payable {
        leasing.createLeaseCrowdloan(
            crowdloanDeposit,
            crowdloanMinContribution,
            crowdloanCap,
            crowdloanEnd,
            leasingEmissionsShare,
            hasLeasingEndBlock,
            leasingEndBlock
        );
    }
}
