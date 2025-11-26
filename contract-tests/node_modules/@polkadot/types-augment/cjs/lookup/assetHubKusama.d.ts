declare const _default: {
    /**
     * Lookup148: asset_hub_kusama_runtime::ProxyType
     **/
    AssetHubKusamaRuntimeProxyType: {
        _enum: string[];
    };
    /**
     * Lookup253: asset_hub_kusama_runtime::RuntimeHoldReason
     **/
    AssetHubKusamaRuntimeRuntimeHoldReason: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            __Unused2: string;
            __Unused3: string;
            __Unused4: string;
            __Unused5: string;
            __Unused6: string;
            __Unused7: string;
            __Unused8: string;
            __Unused9: string;
            __Unused10: string;
            __Unused11: string;
            __Unused12: string;
            __Unused13: string;
            __Unused14: string;
            __Unused15: string;
            __Unused16: string;
            __Unused17: string;
            __Unused18: string;
            __Unused19: string;
            __Unused20: string;
            __Unused21: string;
            __Unused22: string;
            __Unused23: string;
            __Unused24: string;
            __Unused25: string;
            __Unused26: string;
            __Unused27: string;
            __Unused28: string;
            __Unused29: string;
            __Unused30: string;
            PolkadotXcm: string;
            __Unused32: string;
            __Unused33: string;
            __Unused34: string;
            __Unused35: string;
            __Unused36: string;
            __Unused37: string;
            __Unused38: string;
            __Unused39: string;
            __Unused40: string;
            __Unused41: string;
            __Unused42: string;
            __Unused43: string;
            __Unused44: string;
            __Unused45: string;
            __Unused46: string;
            __Unused47: string;
            __Unused48: string;
            __Unused49: string;
            __Unused50: string;
            __Unused51: string;
            __Unused52: string;
            __Unused53: string;
            NftFractionalization: string;
            __Unused55: string;
            __Unused56: string;
            __Unused57: string;
            __Unused58: string;
            __Unused59: string;
            Revive: string;
            __Unused61: string;
            __Unused62: string;
            __Unused63: string;
            __Unused64: string;
            __Unused65: string;
            __Unused66: string;
            __Unused67: string;
            __Unused68: string;
            __Unused69: string;
            StateTrieMigration: string;
        };
    };
    /**
     * Lookup283: asset_hub_kusama_runtime::SessionKeys
     **/
    AssetHubKusamaRuntimeSessionKeys: {
        aura: string;
    };
    /**
     * Lookup284: sp_consensus_aura::sr25519::app_sr25519::Public
     **/
    SpConsensusAuraSr25519AppSr25519Public: string;
    /**
     * Lookup398: pallet_remote_proxy::pallet::Call<T, I>
     **/
    PalletRemoteProxyCall: {
        _enum: {
            remote_proxy: {
                real: string;
                forceProxyType: string;
                call: string;
                proof: string;
            };
            register_remote_proxy_proof: {
                proof: string;
            };
            remote_proxy_with_registered_proof: {
                real: string;
                forceProxyType: string;
                call: string;
            };
        };
    };
    /**
     * Lookup399: pallet_remote_proxy::pallet::RemoteProxyProof<RemoteBlockNumber>
     **/
    PalletRemoteProxyRemoteProxyProof: {
        _enum: {
            RelayChain: {
                proof: string;
                block: string;
            };
        };
    };
    /**
     * Lookup438: asset_hub_kusama_runtime::OriginCaller
     **/
    AssetHubKusamaRuntimeOriginCaller: {
        _enum: {
            system: string;
            __Unused1: string;
            __Unused2: string;
            __Unused3: string;
            __Unused4: string;
            __Unused5: string;
            __Unused6: string;
            __Unused7: string;
            __Unused8: string;
            __Unused9: string;
            __Unused10: string;
            __Unused11: string;
            __Unused12: string;
            __Unused13: string;
            __Unused14: string;
            __Unused15: string;
            __Unused16: string;
            __Unused17: string;
            __Unused18: string;
            __Unused19: string;
            __Unused20: string;
            __Unused21: string;
            __Unused22: string;
            __Unused23: string;
            __Unused24: string;
            __Unused25: string;
            __Unused26: string;
            __Unused27: string;
            __Unused28: string;
            __Unused29: string;
            __Unused30: string;
            PolkadotXcm: string;
            CumulusXcm: string;
        };
    };
    /**
     * Lookup459: pallet_remote_proxy::pallet::Error<T, I>
     **/
    PalletRemoteProxyError: {
        _enum: string[];
    };
    /**
     * Lookup532: asset_hub_kusama_runtime::Runtime
     **/
    AssetHubKusamaRuntimeRuntime: string;
    /**
     * Lookup634: asset_hub_kusama_runtime::RuntimeError
     **/
    AssetHubKusamaRuntimeRuntimeError: {
        _enum: {
            System: string;
            ParachainSystem: string;
            __Unused2: string;
            __Unused3: string;
            __Unused4: string;
            __Unused5: string;
            __Unused6: string;
            __Unused7: string;
            __Unused8: string;
            __Unused9: string;
            Balances: string;
            __Unused11: string;
            __Unused12: string;
            __Unused13: string;
            Vesting: string;
            __Unused15: string;
            __Unused16: string;
            __Unused17: string;
            __Unused18: string;
            __Unused19: string;
            __Unused20: string;
            CollatorSelection: string;
            Session: string;
            __Unused23: string;
            __Unused24: string;
            __Unused25: string;
            __Unused26: string;
            __Unused27: string;
            __Unused28: string;
            __Unused29: string;
            XcmpQueue: string;
            PolkadotXcm: string;
            __Unused32: string;
            __Unused33: string;
            __Unused34: string;
            MessageQueue: string;
            __Unused36: string;
            __Unused37: string;
            __Unused38: string;
            __Unused39: string;
            Utility: string;
            Multisig: string;
            Proxy: string;
            RemoteProxyRelayChain: string;
            __Unused44: string;
            __Unused45: string;
            __Unused46: string;
            __Unused47: string;
            __Unused48: string;
            __Unused49: string;
            Assets: string;
            Uniques: string;
            Nfts: string;
            ForeignAssets: string;
            NftFractionalization: string;
            PoolAssets: string;
            AssetConversion: string;
            __Unused57: string;
            __Unused58: string;
            __Unused59: string;
            Revive: string;
            __Unused61: string;
            __Unused62: string;
            __Unused63: string;
            __Unused64: string;
            __Unused65: string;
            __Unused66: string;
            __Unused67: string;
            __Unused68: string;
            __Unused69: string;
            StateTrieMigration: string;
        };
    };
};
export default _default;
