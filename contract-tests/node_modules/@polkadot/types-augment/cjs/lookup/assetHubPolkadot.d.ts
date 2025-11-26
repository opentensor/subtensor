declare const _default: {
    /**
     * Lookup32: cumulus_pallet_parachain_system::pallet::Event<T>
     **/
    CumulusPalletParachainSystemEvent: {
        _enum: {
            ValidationFunctionStored: string;
            ValidationFunctionApplied: {
                relayChainBlockNum: string;
            };
            ValidationFunctionDiscarded: string;
            DownwardMessagesReceived: {
                count: string;
            };
            DownwardMessagesProcessed: {
                weightUsed: string;
                dmqHead: string;
            };
            UpwardMessageSent: {
                messageHash: string;
            };
        };
    };
    /**
     * Lookup57: pallet_collator_selection::pallet::Event<T>
     **/
    PalletCollatorSelectionEvent: {
        _enum: {
            NewInvulnerables: {
                invulnerables: string;
            };
            InvulnerableAdded: {
                accountId: string;
            };
            InvulnerableRemoved: {
                accountId: string;
            };
            NewDesiredCandidates: {
                desiredCandidates: string;
            };
            NewCandidacyBond: {
                bondAmount: string;
            };
            CandidateAdded: {
                accountId: string;
                deposit: string;
            };
            CandidateBondUpdated: {
                accountId: string;
                deposit: string;
            };
            CandidateRemoved: {
                accountId: string;
            };
            CandidateReplaced: {
                _alias: {
                    new_: string;
                };
                old: string;
                new_: string;
                deposit: string;
            };
            InvalidInvulnerableSkipped: {
                accountId: string;
            };
        };
    };
    /**
     * Lookup60: cumulus_pallet_xcmp_queue::pallet::Event<T>
     **/
    CumulusPalletXcmpQueueEvent: {
        _enum: {
            XcmpMessageSent: {
                messageHash: string;
            };
        };
    };
    /**
     * Lookup135: cumulus_pallet_xcm::pallet::Event<T>
     **/
    CumulusPalletXcmEvent: {
        _enum: {
            InvalidFormat: string;
            UnsupportedVersion: string;
            ExecutedDownward: string;
        };
    };
    /**
     * Lookup136: pallet_xcm_bridge_hub_router::pallet::Event<T, I>
     **/
    PalletXcmBridgeHubRouterEvent: {
        _enum: {
            DeliveryFeeFactorDecreased: {
                newValue: string;
            };
            DeliveryFeeFactorIncreased: {
                newValue: string;
            };
        };
    };
    /**
     * Lookup139: cumulus_primitives_core::AggregateMessageOrigin
     **/
    CumulusPrimitivesCoreAggregateMessageOrigin: {
        _enum: {
            Here: string;
            Parent: string;
            Sibling: string;
        };
    };
    /**
     * Lookup148: asset_hub_polkadot_runtime::ProxyType
     **/
    AssetHubPolkadotRuntimeProxyType: {
        _enum: string[];
    };
    /**
     * Lookup200: cumulus_pallet_parachain_system::unincluded_segment::Ancestor<primitive_types::H256>
     **/
    CumulusPalletParachainSystemUnincludedSegmentAncestor: {
        usedBandwidth: string;
        paraHeadHash: string;
        consumedGoAheadSignal: string;
    };
    /**
     * Lookup201: cumulus_pallet_parachain_system::unincluded_segment::UsedBandwidth
     **/
    CumulusPalletParachainSystemUnincludedSegmentUsedBandwidth: {
        umpMsgCount: string;
        umpTotalBytes: string;
        hrmpOutgoing: string;
    };
    /**
     * Lookup203: cumulus_pallet_parachain_system::unincluded_segment::HrmpChannelUpdate
     **/
    CumulusPalletParachainSystemUnincludedSegmentHrmpChannelUpdate: {
        msgCount: string;
        totalBytes: string;
    };
    /**
     * Lookup209: cumulus_pallet_parachain_system::unincluded_segment::SegmentTracker<primitive_types::H256>
     **/
    CumulusPalletParachainSystemUnincludedSegmentSegmentTracker: {
        usedBandwidth: string;
        hrmpWatermark: string;
        consumedGoAheadSignal: string;
    };
    /**
     * Lookup214: sp_trie::storage_proof::StorageProof
     **/
    SpTrieStorageProof: {
        trieNodes: string;
    };
    /**
     * Lookup216: cumulus_pallet_parachain_system::relay_state_snapshot::MessagingStateSnapshot
     **/
    CumulusPalletParachainSystemRelayStateSnapshotMessagingStateSnapshot: {
        dmqMqcHead: string;
        relayDispatchQueueRemainingCapacity: string;
        ingressChannels: string;
        egressChannels: string;
    };
    /**
     * Lookup217: cumulus_pallet_parachain_system::relay_state_snapshot::RelayDispatchQueueRemainingCapacity
     **/
    CumulusPalletParachainSystemRelayStateSnapshotRelayDispatchQueueRemainingCapacity: {
        remainingCount: string;
        remainingSize: string;
    };
    /**
     * Lookup220: polkadot_primitives::v8::AbridgedHrmpChannel
     **/
    PolkadotPrimitivesV8AbridgedHrmpChannel: {
        maxCapacity: string;
        maxTotalSize: string;
        maxMessageSize: string;
        msgCount: string;
        totalSize: string;
        mqcHead: string;
    };
    /**
     * Lookup221: polkadot_primitives::v8::AbridgedHostConfiguration
     **/
    PolkadotPrimitivesV8AbridgedHostConfiguration: {
        maxCodeSize: string;
        maxHeadDataSize: string;
        maxUpwardQueueCount: string;
        maxUpwardQueueSize: string;
        maxUpwardMessageSize: string;
        maxUpwardMessageNumPerCandidate: string;
        hrmpMaxMessageNumPerCandidate: string;
        validationUpgradeCooldown: string;
        validationUpgradeDelay: string;
        asyncBackingParams: string;
    };
    /**
     * Lookup229: cumulus_pallet_parachain_system::pallet::Call<T>
     **/
    CumulusPalletParachainSystemCall: {
        _enum: {
            set_validation_data: {
                data: string;
            };
            sudo_send_upward_message: {
                message: string;
            };
        };
    };
    /**
     * Lookup230: cumulus_primitives_parachain_inherent::ParachainInherentData
     **/
    CumulusPrimitivesParachainInherentParachainInherentData: {
        validationData: string;
        relayChainState: string;
        downwardMessages: string;
        horizontalMessages: string;
    };
    /**
     * Lookup238: cumulus_pallet_parachain_system::pallet::Error<T>
     **/
    CumulusPalletParachainSystemError: {
        _enum: string[];
    };
    /**
     * Lookup240: staging_parachain_info::pallet::Call<T>
     **/
    StagingParachainInfoCall: string;
    /**
     * Lookup250: asset_hub_polkadot_runtime::RuntimeHoldReason
     **/
    AssetHubPolkadotRuntimeRuntimeHoldReason: {
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
            __Unused54: string;
            __Unused55: string;
            __Unused56: string;
            __Unused57: string;
            __Unused58: string;
            __Unused59: string;
            __Unused60: string;
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
     * Lookup271: pallet_collator_selection::pallet::CandidateInfo<sp_core::crypto::AccountId32, Balance>
     **/
    PalletCollatorSelectionCandidateInfo: {
        who: string;
        deposit: string;
    };
    /**
     * Lookup273: pallet_collator_selection::pallet::Call<T>
     **/
    PalletCollatorSelectionCall: {
        _enum: {
            set_invulnerables: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_desired_candidates: {
                max: string;
            };
            set_candidacy_bond: {
                bond: string;
            };
            register_as_candidate: string;
            leave_intent: string;
            add_invulnerable: {
                who: string;
            };
            remove_invulnerable: {
                who: string;
            };
            update_bond: {
                newDeposit: string;
            };
            take_candidate_slot: {
                deposit: string;
                target: string;
            };
        };
    };
    /**
     * Lookup275: pallet_collator_selection::pallet::Error<T>
     **/
    PalletCollatorSelectionError: {
        _enum: string[];
    };
    /**
     * Lookup278: asset_hub_polkadot_runtime::SessionKeys
     **/
    AssetHubPolkadotRuntimeSessionKeys: {
        aura: string;
    };
    /**
     * Lookup279: sp_consensus_aura::ed25519::app_ed25519::Public
     **/
    SpConsensusAuraEd25519AppEd25519Public: string;
    /**
     * Lookup296: cumulus_pallet_xcmp_queue::OutboundChannelDetails
     **/
    CumulusPalletXcmpQueueOutboundChannelDetails: {
        recipient: string;
        state: string;
        signalsExist: string;
        firstIndex: string;
        lastIndex: string;
    };
    /**
     * Lookup297: cumulus_pallet_xcmp_queue::OutboundState
     **/
    CumulusPalletXcmpQueueOutboundState: {
        _enum: string[];
    };
    /**
     * Lookup301: cumulus_pallet_xcmp_queue::QueueConfigData
     **/
    CumulusPalletXcmpQueueQueueConfigData: {
        suspendThreshold: string;
        dropThreshold: string;
        resumeThreshold: string;
    };
    /**
     * Lookup302: cumulus_pallet_xcmp_queue::pallet::Call<T>
     **/
    CumulusPalletXcmpQueueCall: {
        _enum: {
            __Unused0: string;
            suspend_xcm_execution: string;
            resume_xcm_execution: string;
            update_suspend_threshold: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            update_drop_threshold: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            update_resume_threshold: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
        };
    };
    /**
     * Lookup303: cumulus_pallet_xcmp_queue::pallet::Error<T>
     **/
    CumulusPalletXcmpQueueError: {
        _enum: string[];
    };
    /**
     * Lookup375: cumulus_pallet_xcm::pallet::Call<T>
     **/
    CumulusPalletXcmCall: string;
    /**
     * Lookup376: bp_xcm_bridge_hub_router::BridgeState
     **/
    BpXcmBridgeHubRouterBridgeState: {
        deliveryFeeFactor: string;
        isCongested: string;
    };
    /**
     * Lookup377: pallet_xcm_bridge_hub_router::pallet::Call<T, I>
     **/
    PalletXcmBridgeHubRouterCall: {
        _enum: {
            report_bridge_status: {
                bridgeId: string;
                isCongested: string;
            };
        };
    };
    /**
     * Lookup429: asset_hub_polkadot_runtime::OriginCaller
     **/
    AssetHubPolkadotRuntimeOriginCaller: {
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
     * Lookup432: cumulus_pallet_xcm::pallet::Origin
     **/
    CumulusPalletXcmOrigin: {
        _enum: {
            Relay: string;
            SiblingParachain: string;
        };
    };
    /**
     * Lookup511: asset_hub_polkadot_runtime::Runtime
     **/
    AssetHubPolkadotRuntimeRuntime: string;
    /**
     * Lookup561: assets_common::runtime_api::FungiblesAccessError
     **/
    AssetsCommonRuntimeApiFungiblesAccessError: {
        _enum: string[];
    };
    /**
     * Lookup562: cumulus_primitives_core::CollationInfo
     **/
    CumulusPrimitivesCoreCollationInfo: {
        upwardMessages: string;
        horizontalMessages: string;
        newValidationCode: string;
        processedDownwardMessages: string;
        hrmpWatermark: string;
        headData: string;
    };
    /**
     * Lookup570: asset_hub_polkadot_runtime::RuntimeError
     **/
    AssetHubPolkadotRuntimeRuntimeError: {
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
            __Unused43: string;
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
            PoolAssets: string;
            AssetConversion: string;
            __Unused56: string;
            __Unused57: string;
            __Unused58: string;
            __Unused59: string;
            __Unused60: string;
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
