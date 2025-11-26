declare const _default: {
    /**
     * Lookup57: polkadot_runtime_common::impls::VersionedLocatableAsset
     **/
    PolkadotRuntimeCommonImplsVersionedLocatableAsset: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            __Unused2: string;
            V3: {
                location: string;
                assetId: string;
            };
            V4: {
                location: string;
                assetId: string;
            };
            V5: {
                location: string;
                assetId: string;
            };
        };
    };
    /**
     * Lookup58: staging_xcm::v3::multilocation::MultiLocation
     **/
    StagingXcmV3MultiLocation: {
        parents: string;
        interior: string;
    };
    /**
     * Lookup59: xcm::v3::junctions::Junctions
     **/
    XcmV3Junctions: {
        _enum: {
            Here: string;
            X1: string;
            X2: string;
            X3: string;
            X4: string;
            X5: string;
            X6: string;
            X7: string;
            X8: string;
        };
    };
    /**
     * Lookup60: xcm::v3::junction::Junction
     **/
    XcmV3Junction: {
        _enum: {
            Parachain: string;
            AccountId32: {
                network: string;
                id: string;
            };
            AccountIndex64: {
                network: string;
                index: string;
            };
            AccountKey20: {
                network: string;
                key: string;
            };
            PalletInstance: string;
            GeneralIndex: string;
            GeneralKey: {
                length: string;
                data: string;
            };
            OnlyChild: string;
            Plurality: {
                id: string;
                part: string;
            };
            GlobalConsensus: string;
        };
    };
    /**
     * Lookup63: xcm::v3::junction::NetworkId
     **/
    XcmV3JunctionNetworkId: {
        _enum: {
            ByGenesis: string;
            ByFork: {
                blockNumber: string;
                blockHash: string;
            };
            Polkadot: string;
            Kusama: string;
            Westend: string;
            Rococo: string;
            Wococo: string;
            Ethereum: {
                chainId: string;
            };
            BitcoinCore: string;
            BitcoinCash: string;
            PolkadotBulletin: string;
        };
    };
    /**
     * Lookup66: xcm::v3::junction::BodyId
     **/
    XcmV3JunctionBodyId: {
        _enum: {
            Unit: string;
            Moniker: string;
            Index: string;
            Executive: string;
            Technical: string;
            Legislative: string;
            Judicial: string;
            Defense: string;
            Administration: string;
            Treasury: string;
        };
    };
    /**
     * Lookup67: xcm::v3::junction::BodyPart
     **/
    XcmV3JunctionBodyPart: {
        _enum: {
            Voice: string;
            Members: {
                count: string;
            };
            Fraction: {
                nom: string;
                denom: string;
            };
            AtLeastProportion: {
                nom: string;
                denom: string;
            };
            MoreThanProportion: {
                nom: string;
                denom: string;
            };
        };
    };
    /**
     * Lookup68: xcm::v3::multiasset::AssetId
     **/
    XcmV3MultiassetAssetId: {
        _enum: {
            Concrete: string;
            Abstract: string;
        };
    };
    /**
     * Lookup69: staging_xcm::v4::location::Location
     **/
    StagingXcmV4Location: {
        parents: string;
        interior: string;
    };
    /**
     * Lookup70: staging_xcm::v4::junctions::Junctions
     **/
    StagingXcmV4Junctions: {
        _enum: {
            Here: string;
            X1: string;
            X2: string;
            X3: string;
            X4: string;
            X5: string;
            X6: string;
            X7: string;
            X8: string;
        };
    };
    /**
     * Lookup72: staging_xcm::v4::junction::Junction
     **/
    StagingXcmV4Junction: {
        _enum: {
            Parachain: string;
            AccountId32: {
                network: string;
                id: string;
            };
            AccountIndex64: {
                network: string;
                index: string;
            };
            AccountKey20: {
                network: string;
                key: string;
            };
            PalletInstance: string;
            GeneralIndex: string;
            GeneralKey: {
                length: string;
                data: string;
            };
            OnlyChild: string;
            Plurality: {
                id: string;
                part: string;
            };
            GlobalConsensus: string;
        };
    };
    /**
     * Lookup74: staging_xcm::v4::junction::NetworkId
     **/
    StagingXcmV4JunctionNetworkId: {
        _enum: {
            ByGenesis: string;
            ByFork: {
                blockNumber: string;
                blockHash: string;
            };
            Polkadot: string;
            Kusama: string;
            Westend: string;
            Rococo: string;
            Wococo: string;
            Ethereum: {
                chainId: string;
            };
            BitcoinCore: string;
            BitcoinCash: string;
            PolkadotBulletin: string;
        };
    };
    /**
     * Lookup82: staging_xcm::v4::asset::AssetId
     **/
    StagingXcmV4AssetAssetId: string;
    /**
     * Lookup83: staging_xcm::v5::location::Location
     **/
    StagingXcmV5Location: {
        parents: string;
        interior: string;
    };
    /**
     * Lookup84: staging_xcm::v5::junctions::Junctions
     **/
    StagingXcmV5Junctions: {
        _enum: {
            Here: string;
            X1: string;
            X2: string;
            X3: string;
            X4: string;
            X5: string;
            X6: string;
            X7: string;
            X8: string;
        };
    };
    /**
     * Lookup86: staging_xcm::v5::junction::Junction
     **/
    StagingXcmV5Junction: {
        _enum: {
            Parachain: string;
            AccountId32: {
                network: string;
                id: string;
            };
            AccountIndex64: {
                network: string;
                index: string;
            };
            AccountKey20: {
                network: string;
                key: string;
            };
            PalletInstance: string;
            GeneralIndex: string;
            GeneralKey: {
                length: string;
                data: string;
            };
            OnlyChild: string;
            Plurality: {
                id: string;
                part: string;
            };
            GlobalConsensus: string;
        };
    };
    /**
     * Lookup88: staging_xcm::v5::junction::NetworkId
     **/
    StagingXcmV5JunctionNetworkId: {
        _enum: {
            ByGenesis: string;
            ByFork: {
                blockNumber: string;
                blockHash: string;
            };
            Polkadot: string;
            Kusama: string;
            __Unused4: string;
            __Unused5: string;
            __Unused6: string;
            Ethereum: {
                chainId: string;
            };
            BitcoinCore: string;
            BitcoinCash: string;
            PolkadotBulletin: string;
        };
    };
    /**
     * Lookup96: staging_xcm::v5::asset::AssetId
     **/
    StagingXcmV5AssetAssetId: string;
    /**
     * Lookup97: xcm::VersionedLocation
     **/
    XcmVersionedLocation: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            __Unused2: string;
            V3: string;
            V4: string;
            V5: string;
        };
    };
    /**
     * Lookup145: polkadot_runtime::SessionKeys
     **/
    PolkadotRuntimeSessionKeys: {
        grandpa: string;
        babe: string;
        paraValidator: string;
        paraAssignment: string;
        authorityDiscovery: string;
        beefy: string;
    };
    /**
     * Lookup146: polkadot_primitives::v8::validator_app::Public
     **/
    PolkadotPrimitivesV8ValidatorAppPublic: string;
    /**
     * Lookup147: polkadot_primitives::v8::assignment_app::Public
     **/
    PolkadotPrimitivesV8AssignmentAppPublic: string;
    /**
     * Lookup167: polkadot_runtime::OriginCaller
     **/
    PolkadotRuntimeOriginCaller: {
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
            Origins: string;
            __Unused23: string;
            __Unused24: string;
            __Unused25: string;
            __Unused26: string;
            __Unused27: string;
            __Unused28: string;
            __Unused29: string;
            __Unused30: string;
            __Unused31: string;
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
            ParachainsOrigin: string;
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
            __Unused70: string;
            __Unused71: string;
            __Unused72: string;
            __Unused73: string;
            __Unused74: string;
            __Unused75: string;
            __Unused76: string;
            __Unused77: string;
            __Unused78: string;
            __Unused79: string;
            __Unused80: string;
            __Unused81: string;
            __Unused82: string;
            __Unused83: string;
            __Unused84: string;
            __Unused85: string;
            __Unused86: string;
            __Unused87: string;
            __Unused88: string;
            __Unused89: string;
            __Unused90: string;
            __Unused91: string;
            __Unused92: string;
            __Unused93: string;
            __Unused94: string;
            __Unused95: string;
            __Unused96: string;
            __Unused97: string;
            __Unused98: string;
            XcmPallet: string;
        };
    };
    /**
     * Lookup169: polkadot_runtime::governance::origins::pallet_custom_origins::Origin
     **/
    PolkadotRuntimeGovernanceOriginsPalletCustomOriginsOrigin: {
        _enum: string[];
    };
    /**
     * Lookup170: polkadot_runtime_parachains::origin::pallet::Origin
     **/
    PolkadotRuntimeParachainsOriginPalletOrigin: {
        _enum: {
            Parachain: string;
        };
    };
    /**
     * Lookup172: pallet_xcm::pallet::Origin
     **/
    PalletXcmOrigin: {
        _enum: {
            Xcm: string;
            Response: string;
        };
    };
    /**
     * Lookup176: polkadot_runtime_common::claims::pallet::Call<T>
     **/
    PolkadotRuntimeCommonClaimsPalletCall: {
        _enum: {
            claim: {
                dest: string;
                ethereumSignature: string;
            };
            mint_claim: {
                who: string;
                value: string;
                vestingSchedule: string;
                statement: string;
            };
            claim_attest: {
                dest: string;
                ethereumSignature: string;
                statement: string;
            };
            attest: {
                statement: string;
            };
            move_claim: {
                _alias: {
                    new_: string;
                };
                old: string;
                new_: string;
                maybePreclaim: string;
            };
        };
    };
    /**
     * Lookup177: polkadot_runtime_common::claims::EcdsaSignature
     **/
    PolkadotRuntimeCommonClaimsEcdsaSignature: string;
    /**
     * Lookup183: polkadot_runtime_common::claims::StatementKind
     **/
    PolkadotRuntimeCommonClaimsStatementKind: {
        _enum: string[];
    };
    /**
     * Lookup190: polkadot_runtime_constants::proxy::ProxyType
     **/
    PolkadotRuntimeConstantsProxyProxyType: {
        _enum: string[];
    };
    /**
     * Lookup198: polkadot_runtime::NposCompactSolution16
     **/
    PolkadotRuntimeNposCompactSolution16: {
        votes1: string;
        votes2: string;
        votes3: string;
        votes4: string;
        votes5: string;
        votes6: string;
        votes7: string;
        votes8: string;
        votes9: string;
        votes10: string;
        votes11: string;
        votes12: string;
        votes13: string;
        votes14: string;
        votes15: string;
        votes16: string;
    };
    /**
     * Lookup272: polkadot_runtime_parachains::configuration::pallet::Call<T>
     **/
    PolkadotRuntimeParachainsConfigurationPalletCall: {
        _enum: {
            set_validation_upgrade_cooldown: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_validation_upgrade_delay: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_code_retention_period: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_max_code_size: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_max_pov_size: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_max_head_data_size: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_coretime_cores: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            __Unused7: string;
            set_group_rotation_frequency: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_paras_availability_period: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            __Unused10: string;
            set_scheduling_lookahead: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_max_validators_per_core: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_max_validators: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_dispute_period: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_dispute_post_conclusion_acceptance_period: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            __Unused16: string;
            __Unused17: string;
            set_no_show_slots: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_n_delay_tranches: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_zeroth_delay_tranche_width: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_needed_approvals: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_relay_vrf_modulo_samples: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_max_upward_queue_count: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_max_upward_queue_size: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_max_downward_message_size: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            __Unused26: string;
            set_max_upward_message_size: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_max_upward_message_num_per_candidate: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_hrmp_open_request_ttl: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_hrmp_sender_deposit: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_hrmp_recipient_deposit: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_hrmp_channel_max_capacity: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_hrmp_channel_max_total_size: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_hrmp_max_parachain_inbound_channels: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            __Unused35: string;
            set_hrmp_channel_max_message_size: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_hrmp_max_parachain_outbound_channels: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            __Unused38: string;
            set_hrmp_max_message_num_per_candidate: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            __Unused40: string;
            __Unused41: string;
            set_pvf_voting_ttl: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_minimum_validation_upgrade_delay: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_bypass_consistency_check: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_async_backing_params: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_executor_params: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_on_demand_base_fee: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_on_demand_fee_variability: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_on_demand_queue_max_size: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_on_demand_target_queue_utilization: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            __Unused51: string;
            set_minimum_backing_votes: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_node_feature: {
                index: string;
                value: string;
            };
            set_approval_voting_params: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_scheduler_params: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
        };
    };
    /**
     * Lookup273: polkadot_primitives::v8::async_backing::AsyncBackingParams
     **/
    PolkadotPrimitivesV8AsyncBackingAsyncBackingParams: {
        maxCandidateDepth: string;
        allowedAncestryLen: string;
    };
    /**
     * Lookup274: polkadot_primitives::v8::executor_params::ExecutorParams
     **/
    PolkadotPrimitivesV8ExecutorParams: string;
    /**
     * Lookup276: polkadot_primitives::v8::executor_params::ExecutorParam
     **/
    PolkadotPrimitivesV8ExecutorParamsExecutorParam: {
        _enum: {
            __Unused0: string;
            MaxMemoryPages: string;
            StackLogicalMax: string;
            StackNativeMax: string;
            PrecheckingMaxMemory: string;
            PvfPrepTimeout: string;
            PvfExecTimeout: string;
            WasmExtBulkMemory: string;
        };
    };
    /**
     * Lookup277: polkadot_primitives::v8::PvfPrepKind
     **/
    PolkadotPrimitivesV8PvfPrepKind: {
        _enum: string[];
    };
    /**
     * Lookup278: polkadot_primitives::v8::PvfExecKind
     **/
    PolkadotPrimitivesV8PvfExecKind: {
        _enum: string[];
    };
    /**
     * Lookup279: polkadot_primitives::v8::ApprovalVotingParams
     **/
    PolkadotPrimitivesV8ApprovalVotingParams: {
        maxApprovalCoalesceCount: string;
    };
    /**
     * Lookup280: polkadot_primitives::v8::SchedulerParams<BlockNumber>
     **/
    PolkadotPrimitivesV8SchedulerParams: {
        groupRotationFrequency: string;
        parasAvailabilityPeriod: string;
        maxValidatorsPerCore: string;
        lookahead: string;
        numCores: string;
        maxAvailabilityTimeouts: string;
        onDemandQueueMaxSize: string;
        onDemandTargetQueueUtilization: string;
        onDemandFeeVariability: string;
        onDemandBaseFee: string;
        ttl: string;
    };
    /**
     * Lookup281: polkadot_runtime_parachains::shared::pallet::Call<T>
     **/
    PolkadotRuntimeParachainsSharedPalletCall: string;
    /**
     * Lookup282: polkadot_runtime_parachains::inclusion::pallet::Call<T>
     **/
    PolkadotRuntimeParachainsInclusionPalletCall: string;
    /**
     * Lookup283: polkadot_runtime_parachains::paras_inherent::pallet::Call<T>
     **/
    PolkadotRuntimeParachainsParasInherentPalletCall: {
        _enum: {
            enter: {
                data: string;
            };
        };
    };
    /**
     * Lookup284: polkadot_primitives::vstaging::InherentData<sp_runtime::generic::header::Header<Number, Hash>>
     **/
    PolkadotPrimitivesVstagingInherentData: {
        bitfields: string;
        backedCandidates: string;
        disputes: string;
        parentHeader: string;
    };
    /**
     * Lookup286: polkadot_primitives::v8::signed::UncheckedSigned<polkadot_primitives::v8::AvailabilityBitfield, polkadot_primitives::v8::AvailabilityBitfield>
     **/
    PolkadotPrimitivesV8SignedUncheckedSigned: {
        payload: string;
        validatorIndex: string;
        signature: string;
    };
    /**
     * Lookup289: bitvec::order::Lsb0
     **/
    BitvecOrderLsb0: string;
    /**
     * Lookup291: polkadot_primitives::v8::validator_app::Signature
     **/
    PolkadotPrimitivesV8ValidatorAppSignature: string;
    /**
     * Lookup293: polkadot_primitives::vstaging::BackedCandidate<primitive_types::H256>
     **/
    PolkadotPrimitivesVstagingBackedCandidate: {
        candidate: string;
        validityVotes: string;
        validatorIndices: string;
    };
    /**
     * Lookup294: polkadot_primitives::vstaging::CommittedCandidateReceiptV2<primitive_types::H256>
     **/
    PolkadotPrimitivesVstagingCommittedCandidateReceiptV2: {
        descriptor: string;
        commitments: string;
    };
    /**
     * Lookup295: polkadot_primitives::vstaging::CandidateDescriptorV2<primitive_types::H256>
     **/
    PolkadotPrimitivesVstagingCandidateDescriptorV2: {
        paraId: string;
        relayParent: string;
        version: string;
        coreIndex: string;
        sessionIndex: string;
        reserved1: string;
        persistedValidationDataHash: string;
        povHash: string;
        erasureRoot: string;
        reserved2: string;
        paraHead: string;
        validationCodeHash: string;
    };
    /**
     * Lookup299: polkadot_primitives::v8::CandidateCommitments<N>
     **/
    PolkadotPrimitivesV8CandidateCommitments: {
        upwardMessages: string;
        horizontalMessages: string;
        newValidationCode: string;
        headData: string;
        processedDownwardMessages: string;
        hrmpWatermark: string;
    };
    /**
     * Lookup302: polkadot_core_primitives::OutboundHrmpMessage<polkadot_parachain_primitives::primitives::Id>
     **/
    PolkadotCorePrimitivesOutboundHrmpMessage: {
        recipient: string;
        data: string;
    };
    /**
     * Lookup308: polkadot_primitives::v8::ValidityAttestation
     **/
    PolkadotPrimitivesV8ValidityAttestation: {
        _enum: {
            __Unused0: string;
            Implicit: string;
            Explicit: string;
        };
    };
    /**
     * Lookup310: polkadot_primitives::v8::DisputeStatementSet
     **/
    PolkadotPrimitivesV8DisputeStatementSet: {
        candidateHash: string;
        session: string;
        statements: string;
    };
    /**
     * Lookup314: polkadot_primitives::v8::DisputeStatement
     **/
    PolkadotPrimitivesV8DisputeStatement: {
        _enum: {
            Valid: string;
            Invalid: string;
        };
    };
    /**
     * Lookup315: polkadot_primitives::v8::ValidDisputeStatementKind
     **/
    PolkadotPrimitivesV8ValidDisputeStatementKind: {
        _enum: {
            Explicit: string;
            BackingSeconded: string;
            BackingValid: string;
            ApprovalChecking: string;
            ApprovalCheckingMultipleCandidates: string;
        };
    };
    /**
     * Lookup317: polkadot_primitives::v8::InvalidDisputeStatementKind
     **/
    PolkadotPrimitivesV8InvalidDisputeStatementKind: {
        _enum: string[];
    };
    /**
     * Lookup318: polkadot_runtime_parachains::paras::pallet::Call<T>
     **/
    PolkadotRuntimeParachainsParasPalletCall: {
        _enum: {
            force_set_current_code: {
                para: string;
                newCode: string;
            };
            force_set_current_head: {
                para: string;
                newHead: string;
            };
            force_schedule_code_upgrade: {
                para: string;
                newCode: string;
                relayParentNumber: string;
            };
            force_note_new_head: {
                para: string;
                newHead: string;
            };
            force_queue_action: {
                para: string;
            };
            add_trusted_validation_code: {
                validationCode: string;
            };
            poke_unused_validation_code: {
                validationCodeHash: string;
            };
            include_pvf_check_statement: {
                stmt: string;
                signature: string;
            };
            force_set_most_recent_context: {
                para: string;
                context: string;
            };
        };
    };
    /**
     * Lookup319: polkadot_primitives::v8::PvfCheckStatement
     **/
    PolkadotPrimitivesV8PvfCheckStatement: {
        accept: string;
        subject: string;
        sessionIndex: string;
        validatorIndex: string;
    };
    /**
     * Lookup320: polkadot_runtime_parachains::initializer::pallet::Call<T>
     **/
    PolkadotRuntimeParachainsInitializerPalletCall: {
        _enum: {
            force_approve: {
                upTo: string;
            };
        };
    };
    /**
     * Lookup321: polkadot_runtime_parachains::hrmp::pallet::Call<T>
     **/
    PolkadotRuntimeParachainsHrmpPalletCall: {
        _enum: {
            hrmp_init_open_channel: {
                recipient: string;
                proposedMaxCapacity: string;
                proposedMaxMessageSize: string;
            };
            hrmp_accept_open_channel: {
                sender: string;
            };
            hrmp_close_channel: {
                channelId: string;
            };
            force_clean_hrmp: {
                para: string;
                numInbound: string;
                numOutbound: string;
            };
            force_process_hrmp_open: {
                channels: string;
            };
            force_process_hrmp_close: {
                channels: string;
            };
            hrmp_cancel_open_request: {
                channelId: string;
                openRequests: string;
            };
            force_open_hrmp_channel: {
                sender: string;
                recipient: string;
                maxCapacity: string;
                maxMessageSize: string;
            };
            establish_system_channel: {
                sender: string;
                recipient: string;
            };
            poke_channel_deposits: {
                sender: string;
                recipient: string;
            };
            establish_channel_with_system: {
                targetSystemChain: string;
            };
        };
    };
    /**
     * Lookup322: polkadot_parachain_primitives::primitives::HrmpChannelId
     **/
    PolkadotParachainPrimitivesPrimitivesHrmpChannelId: {
        sender: string;
        recipient: string;
    };
    /**
     * Lookup323: polkadot_runtime_parachains::disputes::pallet::Call<T>
     **/
    PolkadotRuntimeParachainsDisputesPalletCall: {
        _enum: string[];
    };
    /**
     * Lookup324: polkadot_runtime_parachains::disputes::slashing::pallet::Call<T>
     **/
    PolkadotRuntimeParachainsDisputesSlashingPalletCall: {
        _enum: {
            report_dispute_lost_unsigned: {
                disputeProof: string;
                keyOwnerProof: string;
            };
        };
    };
    /**
     * Lookup325: polkadot_primitives::v8::slashing::DisputeProof
     **/
    PolkadotPrimitivesV8SlashingDisputeProof: {
        timeSlot: string;
        kind: string;
        validatorIndex: string;
        validatorId: string;
    };
    /**
     * Lookup326: polkadot_primitives::v8::slashing::DisputesTimeSlot
     **/
    PolkadotPrimitivesV8SlashingDisputesTimeSlot: {
        sessionIndex: string;
        candidateHash: string;
    };
    /**
     * Lookup327: polkadot_primitives::v8::slashing::SlashingOffenceKind
     **/
    PolkadotPrimitivesV8SlashingSlashingOffenceKind: {
        _enum: string[];
    };
    /**
     * Lookup328: polkadot_runtime_parachains::on_demand::pallet::Call<T>
     **/
    PolkadotRuntimeParachainsOnDemandPalletCall: {
        _enum: {
            place_order_allow_death: {
                maxAmount: string;
                paraId: string;
            };
            place_order_keep_alive: {
                maxAmount: string;
                paraId: string;
            };
            place_order_with_credits: {
                maxAmount: string;
                paraId: string;
            };
        };
    };
    /**
     * Lookup329: polkadot_runtime_common::paras_registrar::pallet::Call<T>
     **/
    PolkadotRuntimeCommonParasRegistrarPalletCall: {
        _enum: {
            register: {
                id: string;
                genesisHead: string;
                validationCode: string;
            };
            force_register: {
                who: string;
                deposit: string;
                id: string;
                genesisHead: string;
                validationCode: string;
            };
            deregister: {
                id: string;
            };
            swap: {
                id: string;
                other: string;
            };
            remove_lock: {
                para: string;
            };
            reserve: string;
            add_lock: {
                para: string;
            };
            schedule_code_upgrade: {
                para: string;
                newCode: string;
            };
            set_current_head: {
                para: string;
                newHead: string;
            };
        };
    };
    /**
     * Lookup330: polkadot_runtime_common::slots::pallet::Call<T>
     **/
    PolkadotRuntimeCommonSlotsPalletCall: {
        _enum: {
            force_lease: {
                para: string;
                leaser: string;
                amount: string;
                periodBegin: string;
                periodCount: string;
            };
            clear_all_leases: {
                para: string;
            };
            trigger_onboard: {
                para: string;
            };
        };
    };
    /**
     * Lookup331: polkadot_runtime_common::auctions::pallet::Call<T>
     **/
    PolkadotRuntimeCommonAuctionsPalletCall: {
        _enum: {
            new_auction: {
                duration: string;
                leasePeriodIndex: string;
            };
            bid: {
                para: string;
                auctionIndex: string;
                firstSlot: string;
                lastSlot: string;
                amount: string;
            };
            cancel_auction: string;
        };
    };
    /**
     * Lookup333: polkadot_runtime_common::crowdloan::pallet::Call<T>
     **/
    PolkadotRuntimeCommonCrowdloanPalletCall: {
        _enum: {
            create: {
                index: string;
                cap: string;
                firstPeriod: string;
                lastPeriod: string;
                end: string;
                verifier: string;
            };
            contribute: {
                index: string;
                value: string;
                signature: string;
            };
            withdraw: {
                who: string;
                index: string;
            };
            refund: {
                index: string;
            };
            dissolve: {
                index: string;
            };
            edit: {
                index: string;
                cap: string;
                firstPeriod: string;
                lastPeriod: string;
                end: string;
                verifier: string;
            };
            add_memo: {
                index: string;
                memo: string;
            };
            poke: {
                index: string;
            };
            contribute_all: {
                index: string;
                signature: string;
            };
        };
    };
    /**
     * Lookup335: sp_runtime::MultiSigner
     **/
    SpRuntimeMultiSigner: {
        _enum: {
            Ed25519: string;
            Sr25519: string;
            Ecdsa: string;
        };
    };
    /**
     * Lookup338: polkadot_runtime_parachains::coretime::pallet::Call<T>
     **/
    PolkadotRuntimeParachainsCoretimePalletCall: {
        _enum: {
            __Unused0: string;
            request_core_count: {
                count: string;
            };
            request_revenue_at: {
                when: string;
            };
            credit_account: {
                who: string;
                amount: string;
            };
            assign_core: {
                core: string;
                begin: string;
                assignment: string;
                endHint: string;
            };
        };
    };
    /**
     * Lookup349: pallet_xcm::pallet::Call<T>
     **/
    PalletXcmCall: {
        _enum: {
            send: {
                dest: string;
                message: string;
            };
            teleport_assets: {
                dest: string;
                beneficiary: string;
                assets: string;
                feeAssetItem: string;
            };
            reserve_transfer_assets: {
                dest: string;
                beneficiary: string;
                assets: string;
                feeAssetItem: string;
            };
            execute: {
                message: string;
                maxWeight: string;
            };
            force_xcm_version: {
                location: string;
                version: string;
            };
            force_default_xcm_version: {
                maybeXcmVersion: string;
            };
            force_subscribe_version_notify: {
                location: string;
            };
            force_unsubscribe_version_notify: {
                location: string;
            };
            limited_reserve_transfer_assets: {
                dest: string;
                beneficiary: string;
                assets: string;
                feeAssetItem: string;
                weightLimit: string;
            };
            limited_teleport_assets: {
                dest: string;
                beneficiary: string;
                assets: string;
                feeAssetItem: string;
                weightLimit: string;
            };
            force_suspension: {
                suspended: string;
            };
            transfer_assets: {
                dest: string;
                beneficiary: string;
                assets: string;
                feeAssetItem: string;
                weightLimit: string;
            };
            claim_assets: {
                assets: string;
                beneficiary: string;
            };
            transfer_assets_using_type_and_then: {
                dest: string;
                assets: string;
                assetsTransferType: string;
                remoteFeesId: string;
                feesTransferType: string;
                customXcmOnDest: string;
                weightLimit: string;
            };
            add_authorized_alias: {
                aliaser: string;
                expires: string;
            };
            remove_authorized_alias: {
                aliaser: string;
            };
            remove_all_authorized_aliases: string;
        };
    };
    /**
     * Lookup350: xcm::VersionedXcm<RuntimeCall>
     **/
    XcmVersionedXcm: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            __Unused2: string;
            V3: string;
            V4: string;
            V5: string;
        };
    };
    /**
     * Lookup351: xcm::v3::Xcm<Call>
     **/
    XcmV3Xcm: string;
    /**
     * Lookup353: xcm::v3::Instruction<Call>
     **/
    XcmV3Instruction: {
        _enum: {
            WithdrawAsset: string;
            ReserveAssetDeposited: string;
            ReceiveTeleportedAsset: string;
            QueryResponse: {
                queryId: string;
                response: string;
                maxWeight: string;
                querier: string;
            };
            TransferAsset: {
                assets: string;
                beneficiary: string;
            };
            TransferReserveAsset: {
                assets: string;
                dest: string;
                xcm: string;
            };
            Transact: {
                originKind: string;
                requireWeightAtMost: string;
                call: string;
            };
            HrmpNewChannelOpenRequest: {
                sender: string;
                maxMessageSize: string;
                maxCapacity: string;
            };
            HrmpChannelAccepted: {
                recipient: string;
            };
            HrmpChannelClosing: {
                initiator: string;
                sender: string;
                recipient: string;
            };
            ClearOrigin: string;
            DescendOrigin: string;
            ReportError: string;
            DepositAsset: {
                assets: string;
                beneficiary: string;
            };
            DepositReserveAsset: {
                assets: string;
                dest: string;
                xcm: string;
            };
            ExchangeAsset: {
                give: string;
                want: string;
                maximal: string;
            };
            InitiateReserveWithdraw: {
                assets: string;
                reserve: string;
                xcm: string;
            };
            InitiateTeleport: {
                assets: string;
                dest: string;
                xcm: string;
            };
            ReportHolding: {
                responseInfo: string;
                assets: string;
            };
            BuyExecution: {
                fees: string;
                weightLimit: string;
            };
            RefundSurplus: string;
            SetErrorHandler: string;
            SetAppendix: string;
            ClearError: string;
            ClaimAsset: {
                assets: string;
                ticket: string;
            };
            Trap: string;
            SubscribeVersion: {
                queryId: string;
                maxResponseWeight: string;
            };
            UnsubscribeVersion: string;
            BurnAsset: string;
            ExpectAsset: string;
            ExpectOrigin: string;
            ExpectError: string;
            ExpectTransactStatus: string;
            QueryPallet: {
                moduleName: string;
                responseInfo: string;
            };
            ExpectPallet: {
                index: string;
                name: string;
                moduleName: string;
                crateMajor: string;
                minCrateMinor: string;
            };
            ReportTransactStatus: string;
            ClearTransactStatus: string;
            UniversalOrigin: string;
            ExportMessage: {
                network: string;
                destination: string;
                xcm: string;
            };
            LockAsset: {
                asset: string;
                unlocker: string;
            };
            UnlockAsset: {
                asset: string;
                target: string;
            };
            NoteUnlockable: {
                asset: string;
                owner: string;
            };
            RequestUnlock: {
                asset: string;
                locker: string;
            };
            SetFeesMode: {
                jitWithdraw: string;
            };
            SetTopic: string;
            ClearTopic: string;
            AliasOrigin: string;
            UnpaidExecution: {
                weightLimit: string;
                checkOrigin: string;
            };
        };
    };
    /**
     * Lookup354: xcm::v3::multiasset::MultiAssets
     **/
    XcmV3MultiassetMultiAssets: string;
    /**
     * Lookup356: xcm::v3::multiasset::MultiAsset
     **/
    XcmV3MultiAsset: {
        id: string;
        fun: string;
    };
    /**
     * Lookup357: xcm::v3::multiasset::Fungibility
     **/
    XcmV3MultiassetFungibility: {
        _enum: {
            Fungible: string;
            NonFungible: string;
        };
    };
    /**
     * Lookup358: xcm::v3::multiasset::AssetInstance
     **/
    XcmV3MultiassetAssetInstance: {
        _enum: {
            Undefined: string;
            Index: string;
            Array4: string;
            Array8: string;
            Array16: string;
            Array32: string;
        };
    };
    /**
     * Lookup360: xcm::v3::Response
     **/
    XcmV3Response: {
        _enum: {
            Null: string;
            Assets: string;
            ExecutionResult: string;
            Version: string;
            PalletsInfo: string;
            DispatchResult: string;
        };
    };
    /**
     * Lookup363: xcm::v3::traits::Error
     **/
    XcmV3TraitsError: {
        _enum: {
            Overflow: string;
            Unimplemented: string;
            UntrustedReserveLocation: string;
            UntrustedTeleportLocation: string;
            LocationFull: string;
            LocationNotInvertible: string;
            BadOrigin: string;
            InvalidLocation: string;
            AssetNotFound: string;
            FailedToTransactAsset: string;
            NotWithdrawable: string;
            LocationCannotHold: string;
            ExceedsMaxMessageSize: string;
            DestinationUnsupported: string;
            Transport: string;
            Unroutable: string;
            UnknownClaim: string;
            FailedToDecode: string;
            MaxWeightInvalid: string;
            NotHoldingFees: string;
            TooExpensive: string;
            Trap: string;
            ExpectationFalse: string;
            PalletNotFound: string;
            NameMismatch: string;
            VersionIncompatible: string;
            HoldingWouldOverflow: string;
            ExportError: string;
            ReanchorFailed: string;
            NoDeal: string;
            FeesNotMet: string;
            LockError: string;
            NoPermission: string;
            Unanchored: string;
            NotDepositable: string;
            UnhandledXcmVersion: string;
            WeightLimitReached: string;
            Barrier: string;
            WeightNotComputable: string;
            ExceedsStackLimit: string;
        };
    };
    /**
     * Lookup365: xcm::v3::PalletInfo
     **/
    XcmV3PalletInfo: {
        index: string;
        name: string;
        moduleName: string;
        major: string;
        minor: string;
        patch: string;
    };
    /**
     * Lookup368: xcm::v3::MaybeErrorCode
     **/
    XcmV3MaybeErrorCode: {
        _enum: {
            Success: string;
            Error: string;
            TruncatedError: string;
        };
    };
    /**
     * Lookup371: xcm::v3::OriginKind
     **/
    XcmV3OriginKind: {
        _enum: string[];
    };
    /**
     * Lookup372: xcm::double_encoded::DoubleEncoded<T>
     **/
    XcmDoubleEncoded: {
        encoded: string;
    };
    /**
     * Lookup373: xcm::v3::QueryResponseInfo
     **/
    XcmV3QueryResponseInfo: {
        destination: string;
        queryId: string;
        maxWeight: string;
    };
    /**
     * Lookup374: xcm::v3::multiasset::MultiAssetFilter
     **/
    XcmV3MultiassetMultiAssetFilter: {
        _enum: {
            Definite: string;
            Wild: string;
        };
    };
    /**
     * Lookup375: xcm::v3::multiasset::WildMultiAsset
     **/
    XcmV3MultiassetWildMultiAsset: {
        _enum: {
            All: string;
            AllOf: {
                id: string;
                fun: string;
            };
            AllCounted: string;
            AllOfCounted: {
                id: string;
                fun: string;
                count: string;
            };
        };
    };
    /**
     * Lookup376: xcm::v3::multiasset::WildFungibility
     **/
    XcmV3MultiassetWildFungibility: {
        _enum: string[];
    };
    /**
     * Lookup377: xcm::v3::WeightLimit
     **/
    XcmV3WeightLimit: {
        _enum: {
            Unlimited: string;
            Limited: string;
        };
    };
    /**
     * Lookup378: staging_xcm::v4::Xcm<Call>
     **/
    StagingXcmV4Xcm: string;
    /**
     * Lookup380: staging_xcm::v4::Instruction<Call>
     **/
    StagingXcmV4Instruction: {
        _enum: {
            WithdrawAsset: string;
            ReserveAssetDeposited: string;
            ReceiveTeleportedAsset: string;
            QueryResponse: {
                queryId: string;
                response: string;
                maxWeight: string;
                querier: string;
            };
            TransferAsset: {
                assets: string;
                beneficiary: string;
            };
            TransferReserveAsset: {
                assets: string;
                dest: string;
                xcm: string;
            };
            Transact: {
                originKind: string;
                requireWeightAtMost: string;
                call: string;
            };
            HrmpNewChannelOpenRequest: {
                sender: string;
                maxMessageSize: string;
                maxCapacity: string;
            };
            HrmpChannelAccepted: {
                recipient: string;
            };
            HrmpChannelClosing: {
                initiator: string;
                sender: string;
                recipient: string;
            };
            ClearOrigin: string;
            DescendOrigin: string;
            ReportError: string;
            DepositAsset: {
                assets: string;
                beneficiary: string;
            };
            DepositReserveAsset: {
                assets: string;
                dest: string;
                xcm: string;
            };
            ExchangeAsset: {
                give: string;
                want: string;
                maximal: string;
            };
            InitiateReserveWithdraw: {
                assets: string;
                reserve: string;
                xcm: string;
            };
            InitiateTeleport: {
                assets: string;
                dest: string;
                xcm: string;
            };
            ReportHolding: {
                responseInfo: string;
                assets: string;
            };
            BuyExecution: {
                fees: string;
                weightLimit: string;
            };
            RefundSurplus: string;
            SetErrorHandler: string;
            SetAppendix: string;
            ClearError: string;
            ClaimAsset: {
                assets: string;
                ticket: string;
            };
            Trap: string;
            SubscribeVersion: {
                queryId: string;
                maxResponseWeight: string;
            };
            UnsubscribeVersion: string;
            BurnAsset: string;
            ExpectAsset: string;
            ExpectOrigin: string;
            ExpectError: string;
            ExpectTransactStatus: string;
            QueryPallet: {
                moduleName: string;
                responseInfo: string;
            };
            ExpectPallet: {
                index: string;
                name: string;
                moduleName: string;
                crateMajor: string;
                minCrateMinor: string;
            };
            ReportTransactStatus: string;
            ClearTransactStatus: string;
            UniversalOrigin: string;
            ExportMessage: {
                network: string;
                destination: string;
                xcm: string;
            };
            LockAsset: {
                asset: string;
                unlocker: string;
            };
            UnlockAsset: {
                asset: string;
                target: string;
            };
            NoteUnlockable: {
                asset: string;
                owner: string;
            };
            RequestUnlock: {
                asset: string;
                locker: string;
            };
            SetFeesMode: {
                jitWithdraw: string;
            };
            SetTopic: string;
            ClearTopic: string;
            AliasOrigin: string;
            UnpaidExecution: {
                weightLimit: string;
                checkOrigin: string;
            };
        };
    };
    /**
     * Lookup381: staging_xcm::v4::asset::Assets
     **/
    StagingXcmV4AssetAssets: string;
    /**
     * Lookup383: staging_xcm::v4::asset::Asset
     **/
    StagingXcmV4Asset: {
        id: string;
        fun: string;
    };
    /**
     * Lookup384: staging_xcm::v4::asset::Fungibility
     **/
    StagingXcmV4AssetFungibility: {
        _enum: {
            Fungible: string;
            NonFungible: string;
        };
    };
    /**
     * Lookup385: staging_xcm::v4::asset::AssetInstance
     **/
    StagingXcmV4AssetAssetInstance: {
        _enum: {
            Undefined: string;
            Index: string;
            Array4: string;
            Array8: string;
            Array16: string;
            Array32: string;
        };
    };
    /**
     * Lookup386: staging_xcm::v4::Response
     **/
    StagingXcmV4Response: {
        _enum: {
            Null: string;
            Assets: string;
            ExecutionResult: string;
            Version: string;
            PalletsInfo: string;
            DispatchResult: string;
        };
    };
    /**
     * Lookup388: staging_xcm::v4::PalletInfo
     **/
    StagingXcmV4PalletInfo: {
        index: string;
        name: string;
        moduleName: string;
        major: string;
        minor: string;
        patch: string;
    };
    /**
     * Lookup392: staging_xcm::v4::QueryResponseInfo
     **/
    StagingXcmV4QueryResponseInfo: {
        destination: string;
        queryId: string;
        maxWeight: string;
    };
    /**
     * Lookup393: staging_xcm::v4::asset::AssetFilter
     **/
    StagingXcmV4AssetAssetFilter: {
        _enum: {
            Definite: string;
            Wild: string;
        };
    };
    /**
     * Lookup394: staging_xcm::v4::asset::WildAsset
     **/
    StagingXcmV4AssetWildAsset: {
        _enum: {
            All: string;
            AllOf: {
                id: string;
                fun: string;
            };
            AllCounted: string;
            AllOfCounted: {
                id: string;
                fun: string;
                count: string;
            };
        };
    };
    /**
     * Lookup395: staging_xcm::v4::asset::WildFungibility
     **/
    StagingXcmV4AssetWildFungibility: {
        _enum: string[];
    };
    /**
     * Lookup396: staging_xcm::v5::Xcm<Call>
     **/
    StagingXcmV5Xcm: string;
    /**
     * Lookup398: staging_xcm::v5::Instruction<Call>
     **/
    StagingXcmV5Instruction: {
        _enum: {
            WithdrawAsset: string;
            ReserveAssetDeposited: string;
            ReceiveTeleportedAsset: string;
            QueryResponse: {
                queryId: string;
                response: string;
                maxWeight: string;
                querier: string;
            };
            TransferAsset: {
                assets: string;
                beneficiary: string;
            };
            TransferReserveAsset: {
                assets: string;
                dest: string;
                xcm: string;
            };
            Transact: {
                originKind: string;
                fallbackMaxWeight: string;
                call: string;
            };
            HrmpNewChannelOpenRequest: {
                sender: string;
                maxMessageSize: string;
                maxCapacity: string;
            };
            HrmpChannelAccepted: {
                recipient: string;
            };
            HrmpChannelClosing: {
                initiator: string;
                sender: string;
                recipient: string;
            };
            ClearOrigin: string;
            DescendOrigin: string;
            ReportError: string;
            DepositAsset: {
                assets: string;
                beneficiary: string;
            };
            DepositReserveAsset: {
                assets: string;
                dest: string;
                xcm: string;
            };
            ExchangeAsset: {
                give: string;
                want: string;
                maximal: string;
            };
            InitiateReserveWithdraw: {
                assets: string;
                reserve: string;
                xcm: string;
            };
            InitiateTeleport: {
                assets: string;
                dest: string;
                xcm: string;
            };
            ReportHolding: {
                responseInfo: string;
                assets: string;
            };
            BuyExecution: {
                fees: string;
                weightLimit: string;
            };
            RefundSurplus: string;
            SetErrorHandler: string;
            SetAppendix: string;
            ClearError: string;
            ClaimAsset: {
                assets: string;
                ticket: string;
            };
            Trap: string;
            SubscribeVersion: {
                queryId: string;
                maxResponseWeight: string;
            };
            UnsubscribeVersion: string;
            BurnAsset: string;
            ExpectAsset: string;
            ExpectOrigin: string;
            ExpectError: string;
            ExpectTransactStatus: string;
            QueryPallet: {
                moduleName: string;
                responseInfo: string;
            };
            ExpectPallet: {
                index: string;
                name: string;
                moduleName: string;
                crateMajor: string;
                minCrateMinor: string;
            };
            ReportTransactStatus: string;
            ClearTransactStatus: string;
            UniversalOrigin: string;
            ExportMessage: {
                network: string;
                destination: string;
                xcm: string;
            };
            LockAsset: {
                asset: string;
                unlocker: string;
            };
            UnlockAsset: {
                asset: string;
                target: string;
            };
            NoteUnlockable: {
                asset: string;
                owner: string;
            };
            RequestUnlock: {
                asset: string;
                locker: string;
            };
            SetFeesMode: {
                jitWithdraw: string;
            };
            SetTopic: string;
            ClearTopic: string;
            AliasOrigin: string;
            UnpaidExecution: {
                weightLimit: string;
                checkOrigin: string;
            };
            PayFees: {
                asset: string;
            };
            InitiateTransfer: {
                destination: string;
                remoteFees: string;
                preserveOrigin: string;
                assets: string;
                remoteXcm: string;
            };
            ExecuteWithOrigin: {
                descendantOrigin: string;
                xcm: string;
            };
            SetHints: {
                hints: string;
            };
        };
    };
    /**
     * Lookup399: staging_xcm::v5::asset::Assets
     **/
    StagingXcmV5AssetAssets: string;
    /**
     * Lookup401: staging_xcm::v5::asset::Asset
     **/
    StagingXcmV5Asset: {
        id: string;
        fun: string;
    };
    /**
     * Lookup402: staging_xcm::v5::asset::Fungibility
     **/
    StagingXcmV5AssetFungibility: {
        _enum: {
            Fungible: string;
            NonFungible: string;
        };
    };
    /**
     * Lookup403: staging_xcm::v5::asset::AssetInstance
     **/
    StagingXcmV5AssetAssetInstance: {
        _enum: {
            Undefined: string;
            Index: string;
            Array4: string;
            Array8: string;
            Array16: string;
            Array32: string;
        };
    };
    /**
     * Lookup404: staging_xcm::v5::Response
     **/
    StagingXcmV5Response: {
        _enum: {
            Null: string;
            Assets: string;
            ExecutionResult: string;
            Version: string;
            PalletsInfo: string;
            DispatchResult: string;
        };
    };
    /**
     * Lookup407: xcm::v5::traits::Error
     **/
    XcmV5TraitsError: {
        _enum: {
            Overflow: string;
            Unimplemented: string;
            UntrustedReserveLocation: string;
            UntrustedTeleportLocation: string;
            LocationFull: string;
            LocationNotInvertible: string;
            BadOrigin: string;
            InvalidLocation: string;
            AssetNotFound: string;
            FailedToTransactAsset: string;
            NotWithdrawable: string;
            LocationCannotHold: string;
            ExceedsMaxMessageSize: string;
            DestinationUnsupported: string;
            Transport: string;
            Unroutable: string;
            UnknownClaim: string;
            FailedToDecode: string;
            MaxWeightInvalid: string;
            NotHoldingFees: string;
            TooExpensive: string;
            Trap: string;
            ExpectationFalse: string;
            PalletNotFound: string;
            NameMismatch: string;
            VersionIncompatible: string;
            HoldingWouldOverflow: string;
            ExportError: string;
            ReanchorFailed: string;
            NoDeal: string;
            FeesNotMet: string;
            LockError: string;
            NoPermission: string;
            Unanchored: string;
            NotDepositable: string;
            TooManyAssets: string;
            UnhandledXcmVersion: string;
            WeightLimitReached: string;
            Barrier: string;
            WeightNotComputable: string;
            ExceedsStackLimit: string;
        };
    };
    /**
     * Lookup409: staging_xcm::v5::PalletInfo
     **/
    StagingXcmV5PalletInfo: {
        index: string;
        name: string;
        moduleName: string;
        major: string;
        minor: string;
        patch: string;
    };
    /**
     * Lookup414: staging_xcm::v5::QueryResponseInfo
     **/
    StagingXcmV5QueryResponseInfo: {
        destination: string;
        queryId: string;
        maxWeight: string;
    };
    /**
     * Lookup415: staging_xcm::v5::asset::AssetFilter
     **/
    StagingXcmV5AssetAssetFilter: {
        _enum: {
            Definite: string;
            Wild: string;
        };
    };
    /**
     * Lookup416: staging_xcm::v5::asset::WildAsset
     **/
    StagingXcmV5AssetWildAsset: {
        _enum: {
            All: string;
            AllOf: {
                id: string;
                fun: string;
            };
            AllCounted: string;
            AllOfCounted: {
                id: string;
                fun: string;
                count: string;
            };
        };
    };
    /**
     * Lookup417: staging_xcm::v5::asset::WildFungibility
     **/
    StagingXcmV5AssetWildFungibility: {
        _enum: string[];
    };
    /**
     * Lookup419: staging_xcm::v5::asset::AssetTransferFilter
     **/
    StagingXcmV5AssetAssetTransferFilter: {
        _enum: {
            Teleport: string;
            ReserveDeposit: string;
            ReserveWithdraw: string;
        };
    };
    /**
     * Lookup424: staging_xcm::v5::Hint
     **/
    StagingXcmV5Hint: {
        _enum: {
            AssetClaimer: {
                location: string;
            };
        };
    };
    /**
     * Lookup426: xcm::VersionedAssets
     **/
    XcmVersionedAssets: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            __Unused2: string;
            V3: string;
            V4: string;
            V5: string;
        };
    };
    /**
     * Lookup438: staging_xcm_executor::traits::asset_transfer::TransferType
     **/
    StagingXcmExecutorAssetTransferTransferType: {
        _enum: {
            Teleport: string;
            LocalReserve: string;
            DestinationReserve: string;
            RemoteReserve: string;
        };
    };
    /**
     * Lookup439: xcm::VersionedAssetId
     **/
    XcmVersionedAssetId: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            __Unused2: string;
            V3: string;
            V4: string;
            V5: string;
        };
    };
    /**
     * Lookup442: polkadot_runtime_parachains::inclusion::AggregateMessageOrigin
     **/
    PolkadotRuntimeParachainsInclusionAggregateMessageOrigin: {
        _enum: {
            Ump: string;
        };
    };
    /**
     * Lookup443: polkadot_runtime_parachains::inclusion::UmpQueueId
     **/
    PolkadotRuntimeParachainsInclusionUmpQueueId: {
        _enum: {
            Para: string;
        };
    };
    /**
     * Lookup467: polkadot_runtime_common::claims::pallet::Event<T>
     **/
    PolkadotRuntimeCommonClaimsPalletEvent: {
        _enum: {
            Claimed: {
                who: string;
                ethereumAddress: string;
                amount: string;
            };
        };
    };
    /**
     * Lookup484: polkadot_runtime_parachains::inclusion::pallet::Event<T>
     **/
    PolkadotRuntimeParachainsInclusionPalletEvent: {
        _enum: {
            CandidateBacked: string;
            CandidateIncluded: string;
            CandidateTimedOut: string;
            UpwardMessagesReceived: {
                from: string;
                count: string;
            };
        };
    };
    /**
     * Lookup485: polkadot_primitives::vstaging::CandidateReceiptV2<primitive_types::H256>
     **/
    PolkadotPrimitivesVstagingCandidateReceiptV2: {
        descriptor: string;
        commitmentsHash: string;
    };
    /**
     * Lookup488: polkadot_runtime_parachains::paras::pallet::Event
     **/
    PolkadotRuntimeParachainsParasPalletEvent: {
        _enum: {
            CurrentCodeUpdated: string;
            CurrentHeadUpdated: string;
            CodeUpgradeScheduled: string;
            NewHeadNoted: string;
            ActionQueued: string;
            PvfCheckStarted: string;
            PvfCheckAccepted: string;
            PvfCheckRejected: string;
        };
    };
    /**
     * Lookup489: polkadot_runtime_parachains::hrmp::pallet::Event<T>
     **/
    PolkadotRuntimeParachainsHrmpPalletEvent: {
        _enum: {
            OpenChannelRequested: {
                sender: string;
                recipient: string;
                proposedMaxCapacity: string;
                proposedMaxMessageSize: string;
            };
            OpenChannelCanceled: {
                byParachain: string;
                channelId: string;
            };
            OpenChannelAccepted: {
                sender: string;
                recipient: string;
            };
            ChannelClosed: {
                byParachain: string;
                channelId: string;
            };
            HrmpChannelForceOpened: {
                sender: string;
                recipient: string;
                proposedMaxCapacity: string;
                proposedMaxMessageSize: string;
            };
            HrmpSystemChannelOpened: {
                sender: string;
                recipient: string;
                proposedMaxCapacity: string;
                proposedMaxMessageSize: string;
            };
            OpenChannelDepositsUpdated: {
                sender: string;
                recipient: string;
            };
        };
    };
    /**
     * Lookup490: polkadot_runtime_parachains::disputes::pallet::Event<T>
     **/
    PolkadotRuntimeParachainsDisputesPalletEvent: {
        _enum: {
            DisputeInitiated: string;
            DisputeConcluded: string;
            Revert: string;
        };
    };
    /**
     * Lookup491: polkadot_runtime_parachains::disputes::DisputeLocation
     **/
    PolkadotRuntimeParachainsDisputesDisputeLocation: {
        _enum: string[];
    };
    /**
     * Lookup492: polkadot_runtime_parachains::disputes::DisputeResult
     **/
    PolkadotRuntimeParachainsDisputesDisputeResult: {
        _enum: string[];
    };
    /**
     * Lookup493: polkadot_runtime_parachains::on_demand::pallet::Event<T>
     **/
    PolkadotRuntimeParachainsOnDemandPalletEvent: {
        _enum: {
            OnDemandOrderPlaced: {
                paraId: string;
                spotPrice: string;
                orderedBy: string;
            };
            SpotPriceSet: {
                spotPrice: string;
            };
            AccountCredited: {
                who: string;
                amount: string;
            };
        };
    };
    /**
     * Lookup494: polkadot_runtime_common::paras_registrar::pallet::Event<T>
     **/
    PolkadotRuntimeCommonParasRegistrarPalletEvent: {
        _enum: {
            Registered: {
                paraId: string;
                manager: string;
            };
            Deregistered: {
                paraId: string;
            };
            Reserved: {
                paraId: string;
                who: string;
            };
            Swapped: {
                paraId: string;
                otherId: string;
            };
        };
    };
    /**
     * Lookup495: polkadot_runtime_common::slots::pallet::Event<T>
     **/
    PolkadotRuntimeCommonSlotsPalletEvent: {
        _enum: {
            NewLeasePeriod: {
                leasePeriod: string;
            };
            Leased: {
                paraId: string;
                leaser: string;
                periodBegin: string;
                periodCount: string;
                extraReserved: string;
                totalAmount: string;
            };
        };
    };
    /**
     * Lookup496: polkadot_runtime_common::auctions::pallet::Event<T>
     **/
    PolkadotRuntimeCommonAuctionsPalletEvent: {
        _enum: {
            AuctionStarted: {
                auctionIndex: string;
                leasePeriod: string;
                ending: string;
            };
            AuctionClosed: {
                auctionIndex: string;
            };
            Reserved: {
                bidder: string;
                extraReserved: string;
                totalAmount: string;
            };
            Unreserved: {
                bidder: string;
                amount: string;
            };
            ReserveConfiscated: {
                paraId: string;
                leaser: string;
                amount: string;
            };
            BidAccepted: {
                bidder: string;
                paraId: string;
                amount: string;
                firstSlot: string;
                lastSlot: string;
            };
            WinningOffset: {
                auctionIndex: string;
                blockNumber: string;
            };
        };
    };
    /**
     * Lookup497: polkadot_runtime_common::crowdloan::pallet::Event<T>
     **/
    PolkadotRuntimeCommonCrowdloanPalletEvent: {
        _enum: {
            Created: {
                paraId: string;
            };
            Contributed: {
                who: string;
                fundIndex: string;
                amount: string;
            };
            Withdrew: {
                who: string;
                fundIndex: string;
                amount: string;
            };
            PartiallyRefunded: {
                paraId: string;
            };
            AllRefunded: {
                paraId: string;
            };
            Dissolved: {
                paraId: string;
            };
            HandleBidResult: {
                paraId: string;
                result: string;
            };
            Edited: {
                paraId: string;
            };
            MemoUpdated: {
                who: string;
                paraId: string;
                memo: string;
            };
            AddedToNewRaise: {
                paraId: string;
            };
        };
    };
    /**
     * Lookup498: polkadot_runtime_parachains::coretime::pallet::Event<T>
     **/
    PolkadotRuntimeParachainsCoretimePalletEvent: {
        _enum: {
            RevenueInfoRequested: {
                when: string;
            };
            CoreAssigned: {
                core: string;
            };
        };
    };
    /**
     * Lookup502: pallet_xcm::pallet::Event<T>
     **/
    PalletXcmEvent: {
        _enum: {
            Attempted: {
                outcome: string;
            };
            Sent: {
                origin: string;
                destination: string;
                message: string;
                messageId: string;
            };
            SendFailed: {
                origin: string;
                destination: string;
                error: string;
                messageId: string;
            };
            ProcessXcmError: {
                origin: string;
                error: string;
                messageId: string;
            };
            UnexpectedResponse: {
                origin: string;
                queryId: string;
            };
            ResponseReady: {
                queryId: string;
                response: string;
            };
            Notified: {
                queryId: string;
                palletIndex: string;
                callIndex: string;
            };
            NotifyOverweight: {
                queryId: string;
                palletIndex: string;
                callIndex: string;
                actualWeight: string;
                maxBudgetedWeight: string;
            };
            NotifyDispatchError: {
                queryId: string;
                palletIndex: string;
                callIndex: string;
            };
            NotifyDecodeFailed: {
                queryId: string;
                palletIndex: string;
                callIndex: string;
            };
            InvalidResponder: {
                origin: string;
                queryId: string;
                expectedLocation: string;
            };
            InvalidResponderVersion: {
                origin: string;
                queryId: string;
            };
            ResponseTaken: {
                queryId: string;
            };
            AssetsTrapped: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
                origin: string;
                assets: string;
            };
            VersionChangeNotified: {
                destination: string;
                result: string;
                cost: string;
                messageId: string;
            };
            SupportedVersionChanged: {
                location: string;
                version: string;
            };
            NotifyTargetSendFail: {
                location: string;
                queryId: string;
                error: string;
            };
            NotifyTargetMigrationFail: {
                location: string;
                queryId: string;
            };
            InvalidQuerierVersion: {
                origin: string;
                queryId: string;
            };
            InvalidQuerier: {
                origin: string;
                queryId: string;
                expectedQuerier: string;
                maybeActualQuerier: string;
            };
            VersionNotifyStarted: {
                destination: string;
                cost: string;
                messageId: string;
            };
            VersionNotifyRequested: {
                destination: string;
                cost: string;
                messageId: string;
            };
            VersionNotifyUnrequested: {
                destination: string;
                cost: string;
                messageId: string;
            };
            FeesPaid: {
                paying: string;
                fees: string;
            };
            AssetsClaimed: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
                origin: string;
                assets: string;
            };
            VersionMigrationFinished: {
                version: string;
            };
            AliasAuthorized: {
                aliaser: string;
                target: string;
                expiry: string;
            };
            AliasAuthorizationRemoved: {
                aliaser: string;
                target: string;
            };
            AliasesAuthorizationsRemoved: {
                target: string;
            };
        };
    };
    /**
     * Lookup503: staging_xcm::v5::traits::Outcome
     **/
    StagingXcmV5TraitsOutcome: {
        _enum: {
            Complete: {
                used: string;
            };
            Incomplete: {
                used: string;
                error: string;
            };
            Error: {
                error: string;
            };
        };
    };
    /**
     * Lookup504: xcm::v3::traits::SendError
     **/
    XcmV3TraitsSendError: {
        _enum: string[];
    };
    /**
     * Lookup567: polkadot_runtime::RuntimeHoldReason
     **/
    PolkadotRuntimeRuntimeHoldReason: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            __Unused2: string;
            __Unused3: string;
            __Unused4: string;
            __Unused5: string;
            __Unused6: string;
            Staking: string;
            __Unused8: string;
            __Unused9: string;
            Preimage: string;
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
            __Unused31: string;
            __Unused32: string;
            __Unused33: string;
            __Unused34: string;
            __Unused35: string;
            __Unused36: string;
            __Unused37: string;
            __Unused38: string;
            __Unused39: string;
            __Unused40: string;
            DelegatedStaking: string;
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
            __Unused70: string;
            __Unused71: string;
            __Unused72: string;
            __Unused73: string;
            __Unused74: string;
            __Unused75: string;
            __Unused76: string;
            __Unused77: string;
            __Unused78: string;
            __Unused79: string;
            __Unused80: string;
            __Unused81: string;
            __Unused82: string;
            __Unused83: string;
            __Unused84: string;
            __Unused85: string;
            __Unused86: string;
            __Unused87: string;
            __Unused88: string;
            __Unused89: string;
            __Unused90: string;
            __Unused91: string;
            __Unused92: string;
            __Unused93: string;
            __Unused94: string;
            __Unused95: string;
            __Unused96: string;
            __Unused97: string;
            StateTrieMigration: string;
            XcmPallet: string;
        };
    };
    /**
     * Lookup572: pallet_xcm::pallet::HoldReason
     **/
    PalletXcmHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup576: polkadot_runtime::RuntimeFreezeReason
     **/
    PolkadotRuntimeRuntimeFreezeReason: {
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
            __Unused31: string;
            __Unused32: string;
            __Unused33: string;
            __Unused34: string;
            __Unused35: string;
            __Unused36: string;
            __Unused37: string;
            __Unused38: string;
            NominationPools: string;
        };
    };
    /**
     * Lookup640: pallet_referenda::types::ReferendumInfo<TrackId, polkadot_runtime::OriginCaller, Moment, frame_support::traits::preimages::Bounded<polkadot_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, Balance, pallet_conviction_voting::types::Tally<Votes, Total>, sp_core::crypto::AccountId32, ScheduleAddress>
     **/
    PalletReferendaReferendumInfo: {
        _enum: {
            Ongoing: string;
            Approved: string;
            Rejected: string;
            Cancelled: string;
            TimedOut: string;
            Killed: string;
        };
    };
    /**
     * Lookup641: pallet_referenda::types::ReferendumStatus<TrackId, polkadot_runtime::OriginCaller, Moment, frame_support::traits::preimages::Bounded<polkadot_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, Balance, pallet_conviction_voting::types::Tally<Votes, Total>, sp_core::crypto::AccountId32, ScheduleAddress>
     **/
    PalletReferendaReferendumStatus: {
        track: string;
        origin: string;
        proposal: string;
        enactment: string;
        submitted: string;
        submissionDeposit: string;
        decisionDeposit: string;
        deciding: string;
        tally: string;
        inQueue: string;
        alarm: string;
    };
    /**
     * Lookup659: polkadot_runtime_common::claims::pallet::Error<T>
     **/
    PolkadotRuntimeCommonClaimsPalletError: {
        _enum: string[];
    };
    /**
     * Lookup724: polkadot_runtime_parachains::configuration::HostConfiguration<BlockNumber>
     **/
    PolkadotRuntimeParachainsConfigurationHostConfiguration: {
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
        maxPovSize: string;
        maxDownwardMessageSize: string;
        hrmpMaxParachainOutboundChannels: string;
        hrmpSenderDeposit: string;
        hrmpRecipientDeposit: string;
        hrmpChannelMaxCapacity: string;
        hrmpChannelMaxTotalSize: string;
        hrmpMaxParachainInboundChannels: string;
        hrmpChannelMaxMessageSize: string;
        executorParams: string;
        codeRetentionPeriod: string;
        maxValidators: string;
        disputePeriod: string;
        disputePostConclusionAcceptancePeriod: string;
        noShowSlots: string;
        nDelayTranches: string;
        zerothDelayTrancheWidth: string;
        neededApprovals: string;
        relayVrfModuloSamples: string;
        pvfVotingTtl: string;
        minimumValidationUpgradeDelay: string;
        minimumBackingVotes: string;
        nodeFeatures: string;
        approvalVotingParams: string;
        schedulerParams: string;
    };
    /**
     * Lookup727: polkadot_runtime_parachains::configuration::pallet::Error<T>
     **/
    PolkadotRuntimeParachainsConfigurationPalletError: {
        _enum: string[];
    };
    /**
     * Lookup730: polkadot_runtime_parachains::shared::AllowedRelayParentsTracker<primitive_types::H256, BlockNumber>
     **/
    PolkadotRuntimeParachainsSharedAllowedRelayParentsTracker: {
        buffer: string;
        latestNumber: string;
    };
    /**
     * Lookup732: polkadot_runtime_parachains::shared::RelayParentInfo<primitive_types::H256>
     **/
    PolkadotRuntimeParachainsSharedRelayParentInfo: {
        relayParent: string;
        stateRoot: string;
        claimQueue: string;
    };
    /**
     * Lookup742: polkadot_runtime_parachains::inclusion::CandidatePendingAvailability<primitive_types::H256, N>
     **/
    PolkadotRuntimeParachainsInclusionCandidatePendingAvailability: {
        _alias: {
            hash_: string;
        };
        core: string;
        hash_: string;
        descriptor: string;
        commitments: string;
        availabilityVotes: string;
        backers: string;
        relayParentNumber: string;
        backedInNumber: string;
        backingGroup: string;
    };
    /**
     * Lookup743: polkadot_runtime_parachains::inclusion::pallet::Error<T>
     **/
    PolkadotRuntimeParachainsInclusionPalletError: {
        _enum: string[];
    };
    /**
     * Lookup744: polkadot_primitives::vstaging::ScrapedOnChainVotes<primitive_types::H256>
     **/
    PolkadotPrimitivesVstagingScrapedOnChainVotes: {
        session: string;
        backingValidatorsPerCandidate: string;
        disputes: string;
    };
    /**
     * Lookup749: polkadot_runtime_parachains::paras_inherent::pallet::Error<T>
     **/
    PolkadotRuntimeParachainsParasInherentPalletError: {
        _enum: string[];
    };
    /**
     * Lookup753: polkadot_runtime_parachains::scheduler::common::Assignment
     **/
    PolkadotRuntimeParachainsSchedulerCommonAssignment: {
        _enum: {
            Pool: {
                paraId: string;
                coreIndex: string;
            };
            Bulk: string;
        };
    };
    /**
     * Lookup756: polkadot_runtime_parachains::paras::PvfCheckActiveVoteState<BlockNumber>
     **/
    PolkadotRuntimeParachainsParasPvfCheckActiveVoteState: {
        votesAccept: string;
        votesReject: string;
        age: string;
        createdAt: string;
        causes: string;
    };
    /**
     * Lookup758: polkadot_runtime_parachains::paras::PvfCheckCause<BlockNumber>
     **/
    PolkadotRuntimeParachainsParasPvfCheckCause: {
        _enum: {
            Onboarding: string;
            Upgrade: {
                id: string;
                includedAt: string;
                upgradeStrategy: string;
            };
        };
    };
    /**
     * Lookup759: polkadot_runtime_parachains::paras::UpgradeStrategy
     **/
    PolkadotRuntimeParachainsParasUpgradeStrategy: {
        _enum: string[];
    };
    /**
     * Lookup762: polkadot_runtime_parachains::paras::ParaLifecycle
     **/
    PolkadotRuntimeParachainsParasParaLifecycle: {
        _enum: string[];
    };
    /**
     * Lookup764: polkadot_runtime_parachains::paras::ParaPastCodeMeta<N>
     **/
    PolkadotRuntimeParachainsParasParaPastCodeMeta: {
        upgradeTimes: string;
        lastPruned: string;
    };
    /**
     * Lookup766: polkadot_runtime_parachains::paras::ReplacementTimes<N>
     **/
    PolkadotRuntimeParachainsParasReplacementTimes: {
        expectedAt: string;
        activatedAt: string;
    };
    /**
     * Lookup768: polkadot_primitives::v8::UpgradeGoAhead
     **/
    PolkadotPrimitivesV8UpgradeGoAhead: {
        _enum: string[];
    };
    /**
     * Lookup769: polkadot_primitives::v8::UpgradeRestriction
     **/
    PolkadotPrimitivesV8UpgradeRestriction: {
        _enum: string[];
    };
    /**
     * Lookup770: polkadot_runtime_parachains::paras::ParaGenesisArgs
     **/
    PolkadotRuntimeParachainsParasParaGenesisArgs: {
        genesisHead: string;
        validationCode: string;
        paraKind: string;
    };
    /**
     * Lookup771: polkadot_runtime_parachains::paras::pallet::Error<T>
     **/
    PolkadotRuntimeParachainsParasPalletError: {
        _enum: string[];
    };
    /**
     * Lookup773: polkadot_runtime_parachains::initializer::BufferedSessionChange
     **/
    PolkadotRuntimeParachainsInitializerBufferedSessionChange: {
        validators: string;
        queued: string;
        sessionIndex: string;
    };
    /**
     * Lookup775: polkadot_core_primitives::InboundDownwardMessage<BlockNumber>
     **/
    PolkadotCorePrimitivesInboundDownwardMessage: {
        sentAt: string;
        msg: string;
    };
    /**
     * Lookup776: polkadot_runtime_parachains::hrmp::HrmpOpenChannelRequest
     **/
    PolkadotRuntimeParachainsHrmpHrmpOpenChannelRequest: {
        confirmed: string;
        age: string;
        senderDeposit: string;
        maxMessageSize: string;
        maxCapacity: string;
        maxTotalSize: string;
    };
    /**
     * Lookup778: polkadot_runtime_parachains::hrmp::HrmpChannel
     **/
    PolkadotRuntimeParachainsHrmpHrmpChannel: {
        maxCapacity: string;
        maxTotalSize: string;
        maxMessageSize: string;
        msgCount: string;
        totalSize: string;
        mqcHead: string;
        senderDeposit: string;
        recipientDeposit: string;
    };
    /**
     * Lookup780: polkadot_core_primitives::InboundHrmpMessage<BlockNumber>
     **/
    PolkadotCorePrimitivesInboundHrmpMessage: {
        sentAt: string;
        data: string;
    };
    /**
     * Lookup783: polkadot_runtime_parachains::hrmp::pallet::Error<T>
     **/
    PolkadotRuntimeParachainsHrmpPalletError: {
        _enum: string[];
    };
    /**
     * Lookup785: polkadot_primitives::v8::SessionInfo
     **/
    PolkadotPrimitivesV8SessionInfo: {
        activeValidatorIndices: string;
        randomSeed: string;
        disputePeriod: string;
        validators: string;
        discoveryKeys: string;
        assignmentKeys: string;
        validatorGroups: string;
        nCores: string;
        zerothDelayTrancheWidth: string;
        relayVrfModuloSamples: string;
        nDelayTranches: string;
        noShowSlots: string;
        neededApprovals: string;
    };
    /**
     * Lookup786: polkadot_primitives::v8::IndexedVec<polkadot_primitives::v8::ValidatorIndex, polkadot_primitives::v8::validator_app::Public>
     **/
    PolkadotPrimitivesV8IndexedVecValidatorIndex: string;
    /**
     * Lookup787: polkadot_primitives::v8::IndexedVec<polkadot_primitives::v8::GroupIndex, V>
     **/
    PolkadotPrimitivesV8IndexedVecGroupIndex: string;
    /**
     * Lookup789: polkadot_primitives::v8::DisputeState<N>
     **/
    PolkadotPrimitivesV8DisputeState: {
        validatorsFor: string;
        validatorsAgainst: string;
        start: string;
        concludedAt: string;
    };
    /**
     * Lookup791: polkadot_runtime_parachains::disputes::pallet::Error<T>
     **/
    PolkadotRuntimeParachainsDisputesPalletError: {
        _enum: string[];
    };
    /**
     * Lookup792: polkadot_primitives::v8::slashing::PendingSlashes
     **/
    PolkadotPrimitivesV8SlashingPendingSlashes: {
        _alias: {
            keys_: string;
        };
        keys_: string;
        kind: string;
    };
    /**
     * Lookup796: polkadot_runtime_parachains::disputes::slashing::pallet::Error<T>
     **/
    PolkadotRuntimeParachainsDisputesSlashingPalletError: {
        _enum: string[];
    };
    /**
     * Lookup797: polkadot_runtime_parachains::on_demand::types::CoreAffinityCount
     **/
    PolkadotRuntimeParachainsOnDemandTypesCoreAffinityCount: {
        coreIndex: string;
        count: string;
    };
    /**
     * Lookup798: polkadot_runtime_parachains::on_demand::types::QueueStatusType
     **/
    PolkadotRuntimeParachainsOnDemandTypesQueueStatusType: {
        traffic: string;
        nextIndex: string;
        smallestIndex: string;
        freedIndices: string;
    };
    /**
     * Lookup800: BinaryHeap<polkadot_runtime_parachains::on_demand::types::ReverseQueueIndex>
     **/
    BinaryHeapReverseQueueIndex: string;
    /**
     * Lookup803: BinaryHeap<polkadot_runtime_parachains::on_demand::types::EnqueuedOrder>
     **/
    BinaryHeapEnqueuedOrder: string;
    /**
     * Lookup804: polkadot_runtime_parachains::on_demand::types::EnqueuedOrder
     **/
    PolkadotRuntimeParachainsOnDemandTypesEnqueuedOrder: {
        paraId: string;
        idx: string;
    };
    /**
     * Lookup808: polkadot_runtime_parachains::on_demand::pallet::Error<T>
     **/
    PolkadotRuntimeParachainsOnDemandPalletError: {
        _enum: string[];
    };
    /**
     * Lookup810: polkadot_runtime_parachains::assigner_coretime::Schedule<N>
     **/
    PolkadotRuntimeParachainsAssignerCoretimeSchedule: {
        assignments: string;
        endHint: string;
        nextSchedule: string;
    };
    /**
     * Lookup811: polkadot_runtime_parachains::assigner_coretime::CoreDescriptor<N>
     **/
    PolkadotRuntimeParachainsAssignerCoretimeCoreDescriptor: {
        queue: string;
        currentWork: string;
    };
    /**
     * Lookup813: polkadot_runtime_parachains::assigner_coretime::QueueDescriptor<N>
     **/
    PolkadotRuntimeParachainsAssignerCoretimeQueueDescriptor: {
        first: string;
        last: string;
    };
    /**
     * Lookup815: polkadot_runtime_parachains::assigner_coretime::WorkState<N>
     **/
    PolkadotRuntimeParachainsAssignerCoretimeWorkState: {
        assignments: string;
        endHint: string;
        pos: string;
        step: string;
    };
    /**
     * Lookup818: polkadot_runtime_parachains::assigner_coretime::AssignmentState
     **/
    PolkadotRuntimeParachainsAssignerCoretimeAssignmentState: {
        ratio: string;
        remaining: string;
    };
    /**
     * Lookup819: polkadot_runtime_parachains::assigner_coretime::pallet::Error<T>
     **/
    PolkadotRuntimeParachainsAssignerCoretimePalletError: {
        _enum: string[];
    };
    /**
     * Lookup820: polkadot_runtime_common::paras_registrar::ParaInfo<sp_core::crypto::AccountId32, Balance>
     **/
    PolkadotRuntimeCommonParasRegistrarParaInfo: {
        manager: string;
        deposit: string;
        locked: string;
    };
    /**
     * Lookup822: polkadot_runtime_common::paras_registrar::pallet::Error<T>
     **/
    PolkadotRuntimeCommonParasRegistrarPalletError: {
        _enum: string[];
    };
    /**
     * Lookup824: polkadot_runtime_common::slots::pallet::Error<T>
     **/
    PolkadotRuntimeCommonSlotsPalletError: {
        _enum: string[];
    };
    /**
     * Lookup829: polkadot_runtime_common::auctions::pallet::Error<T>
     **/
    PolkadotRuntimeCommonAuctionsPalletError: {
        _enum: string[];
    };
    /**
     * Lookup830: polkadot_runtime_common::crowdloan::FundInfo<sp_core::crypto::AccountId32, Balance, BlockNumber, LeasePeriod>
     **/
    PolkadotRuntimeCommonCrowdloanFundInfo: {
        depositor: string;
        verifier: string;
        deposit: string;
        raised: string;
        end: string;
        cap: string;
        lastContribution: string;
        firstPeriod: string;
        lastPeriod: string;
        fundIndex: string;
    };
    /**
     * Lookup831: polkadot_runtime_common::crowdloan::LastContribution<BlockNumber>
     **/
    PolkadotRuntimeCommonCrowdloanLastContribution: {
        _enum: {
            Never: string;
            PreEnding: string;
            Ending: string;
        };
    };
    /**
     * Lookup832: polkadot_runtime_common::crowdloan::pallet::Error<T>
     **/
    PolkadotRuntimeCommonCrowdloanPalletError: {
        _enum: string[];
    };
    /**
     * Lookup833: polkadot_runtime_parachains::coretime::pallet::Error<T>
     **/
    PolkadotRuntimeParachainsCoretimePalletError: {
        _enum: string[];
    };
    /**
     * Lookup834: pallet_xcm::pallet::QueryStatus<BlockNumber>
     **/
    PalletXcmQueryStatus: {
        _enum: {
            Pending: {
                responder: string;
                maybeMatchQuerier: string;
                maybeNotify: string;
                timeout: string;
            };
            VersionNotifier: {
                origin: string;
                isActive: string;
            };
            Ready: {
                response: string;
                at: string;
            };
        };
    };
    /**
     * Lookup838: xcm::VersionedResponse
     **/
    XcmVersionedResponse: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            __Unused2: string;
            V3: string;
            V4: string;
            V5: string;
        };
    };
    /**
     * Lookup844: pallet_xcm::pallet::VersionMigrationStage
     **/
    PalletXcmVersionMigrationStage: {
        _enum: {
            MigrateSupportedVersion: string;
            MigrateVersionNotifiers: string;
            NotifyCurrentTargets: string;
            MigrateAndNotifyOldTargets: string;
        };
    };
    /**
     * Lookup847: pallet_xcm::pallet::RemoteLockedFungibleRecord<ConsumerIdentifier, MaxConsumers>
     **/
    PalletXcmRemoteLockedFungibleRecord: {
        amount: string;
        owner: string;
        locker: string;
        consumers: string;
    };
    /**
     * Lookup854: pallet_xcm::AuthorizedAliasesEntry<frame_support::traits::storage::Disabled, pallet_xcm::pallet::MaxAuthorizedAliases>
     **/
    PalletXcmAuthorizedAliasesEntry: {
        aliasers: string;
        ticket: string;
    };
    /**
     * Lookup855: frame_support::traits::storage::Disabled
     **/
    FrameSupportStorageDisabled: string;
    /**
     * Lookup856: pallet_xcm::pallet::MaxAuthorizedAliases
     **/
    PalletXcmMaxAuthorizedAliases: string;
    /**
     * Lookup858: xcm_runtime_apis::authorized_aliases::OriginAliaser
     **/
    XcmRuntimeApisAuthorizedAliasesOriginAliaser: {
        location: string;
        expiry: string;
    };
    /**
     * Lookup860: pallet_xcm::pallet::Error<T>
     **/
    PalletXcmError: {
        _enum: string[];
    };
    /**
     * Lookup882: pallet_transaction_payment::ChargeTransactionPayment<T>
     **/
    PalletTransactionPaymentChargeTransactionPayment: string;
    /**
     * Lookup883: polkadot_runtime_common::claims::PrevalidateAttests<T>
     **/
    PolkadotRuntimeCommonClaimsPrevalidateAttests: string;
    /**
     * Lookup886: polkadot_runtime::Runtime
     **/
    PolkadotRuntimeRuntime: string;
    /**
     * Lookup887: relay_common::apis::InflationInfo
     **/
    RelayCommonApisInflationInfo: {
        inflation: string;
        nextMint: string;
    };
    /**
     * Lookup910: polkadot_primitives::v8::GroupRotationInfo<N>
     **/
    PolkadotPrimitivesV8GroupRotationInfo: {
        sessionStartBlock: string;
        groupRotationFrequency: string;
        now: string;
    };
    /**
     * Lookup912: polkadot_primitives::vstaging::CoreState<primitive_types::H256, N>
     **/
    PolkadotPrimitivesVstagingCoreState: {
        _enum: {
            Occupied: string;
            Scheduled: string;
            Free: string;
        };
    };
    /**
     * Lookup913: polkadot_primitives::vstaging::OccupiedCore<primitive_types::H256, N>
     **/
    PolkadotPrimitivesVstagingOccupiedCore: {
        nextUpOnAvailable: string;
        occupiedSince: string;
        timeOutAt: string;
        nextUpOnTimeOut: string;
        availability: string;
        groupResponsible: string;
        candidateHash: string;
        candidateDescriptor: string;
    };
    /**
     * Lookup915: polkadot_primitives::v8::ScheduledCore
     **/
    PolkadotPrimitivesV8ScheduledCore: {
        paraId: string;
        collator: string;
    };
    /**
     * Lookup917: polkadot_primitives::v8::collator_app::Public
     **/
    PolkadotPrimitivesV8CollatorAppPublic: string;
    /**
     * Lookup918: polkadot_primitives::v8::OccupiedCoreAssumption
     **/
    PolkadotPrimitivesV8OccupiedCoreAssumption: {
        _enum: string[];
    };
    /**
     * Lookup920: polkadot_primitives::v8::PersistedValidationData<primitive_types::H256, N>
     **/
    PolkadotPrimitivesV8PersistedValidationData: {
        parentHead: string;
        relayParentNumber: string;
        relayParentStorageRoot: string;
        maxPovSize: string;
    };
    /**
     * Lookup925: polkadot_primitives::vstaging::CandidateEvent<primitive_types::H256>
     **/
    PolkadotPrimitivesVstagingCandidateEvent: {
        _enum: {
            CandidateBacked: string;
            CandidateIncluded: string;
            CandidateTimedOut: string;
        };
    };
    /**
     * Lookup941: polkadot_primitives::vstaging::async_backing::BackingState<primitive_types::H256, N>
     **/
    PolkadotPrimitivesVstagingAsyncBackingBackingState: {
        constraints: string;
        pendingAvailability: string;
    };
    /**
     * Lookup942: polkadot_primitives::v8::async_backing::Constraints<N>
     **/
    PolkadotPrimitivesV8AsyncBackingConstraints: {
        minRelayParentNumber: string;
        maxPovSize: string;
        maxCodeSize: string;
        umpRemaining: string;
        umpRemainingBytes: string;
        maxUmpNumPerCandidate: string;
        dmpRemainingMessages: string;
        hrmpInbound: string;
        hrmpChannelsOut: string;
        maxHrmpNumPerCandidate: string;
        requiredParent: string;
        validationCodeHash: string;
        upgradeRestriction: string;
        futureValidationCode: string;
    };
    /**
     * Lookup943: polkadot_primitives::v8::async_backing::InboundHrmpLimitations<N>
     **/
    PolkadotPrimitivesV8AsyncBackingInboundHrmpLimitations: {
        validWatermarks: string;
    };
    /**
     * Lookup946: polkadot_primitives::v8::async_backing::OutboundHrmpChannelLimitations
     **/
    PolkadotPrimitivesV8AsyncBackingOutboundHrmpChannelLimitations: {
        bytesRemaining: string;
        messagesRemaining: string;
    };
    /**
     * Lookup951: polkadot_primitives::vstaging::async_backing::CandidatePendingAvailability<primitive_types::H256, N>
     **/
    PolkadotPrimitivesVstagingAsyncBackingCandidatePendingAvailability: {
        candidateHash: string;
        descriptor: string;
        commitments: string;
        relayParentNumber: string;
        maxPovSize: string;
    };
    /**
     * Lookup957: polkadot_primitives::vstaging::async_backing::Constraints<N>
     **/
    PolkadotPrimitivesVstagingAsyncBackingConstraints: {
        minRelayParentNumber: string;
        maxPovSize: string;
        maxCodeSize: string;
        maxHeadDataSize: string;
        umpRemaining: string;
        umpRemainingBytes: string;
        maxUmpNumPerCandidate: string;
        dmpRemainingMessages: string;
        hrmpInbound: string;
        hrmpChannelsOut: string;
        maxHrmpNumPerCandidate: string;
        requiredParent: string;
        validationCodeHash: string;
        upgradeRestriction: string;
        futureValidationCode: string;
    };
    /**
     * Lookup985: xcm_runtime_apis::fees::Error
     **/
    XcmRuntimeApisFeesError: {
        _enum: string[];
    };
    /**
     * Lookup990: xcm_runtime_apis::dry_run::CallDryRunEffects<polkadot_runtime::RuntimeEvent>
     **/
    XcmRuntimeApisDryRunCallDryRunEffects: {
        executionResult: string;
        emittedEvents: string;
        localXcm: string;
        forwardedXcms: string;
    };
    /**
     * Lookup996: xcm_runtime_apis::dry_run::Error
     **/
    XcmRuntimeApisDryRunError: {
        _enum: string[];
    };
    /**
     * Lookup998: xcm_runtime_apis::dry_run::XcmDryRunEffects<polkadot_runtime::RuntimeEvent>
     **/
    XcmRuntimeApisDryRunXcmDryRunEffects: {
        executionResult: string;
        emittedEvents: string;
        forwardedXcms: string;
    };
    /**
     * Lookup1000: xcm_runtime_apis::conversions::Error
     **/
    XcmRuntimeApisConversionsError: {
        _enum: string[];
    };
    /**
     * Lookup1004: polkadot_runtime::RuntimeError
     **/
    PolkadotRuntimeRuntimeError: {
        _enum: {
            System: string;
            Scheduler: string;
            Babe: string;
            __Unused3: string;
            Indices: string;
            Balances: string;
            __Unused6: string;
            Staking: string;
            __Unused8: string;
            Session: string;
            Preimage: string;
            Grandpa: string;
            __Unused12: string;
            __Unused13: string;
            __Unused14: string;
            __Unused15: string;
            __Unused16: string;
            __Unused17: string;
            __Unused18: string;
            Treasury: string;
            ConvictionVoting: string;
            Referenda: string;
            __Unused22: string;
            Whitelist: string;
            Claims: string;
            Vesting: string;
            Utility: string;
            __Unused27: string;
            __Unused28: string;
            Proxy: string;
            Multisig: string;
            __Unused31: string;
            __Unused32: string;
            __Unused33: string;
            Bounties: string;
            __Unused35: string;
            ElectionProviderMultiPhase: string;
            VoterList: string;
            ChildBounties: string;
            NominationPools: string;
            FastUnstake: string;
            DelegatedStaking: string;
            __Unused42: string;
            __Unused43: string;
            __Unused44: string;
            __Unused45: string;
            __Unused46: string;
            __Unused47: string;
            __Unused48: string;
            __Unused49: string;
            __Unused50: string;
            Configuration: string;
            __Unused52: string;
            ParaInclusion: string;
            ParaInherent: string;
            __Unused55: string;
            Paras: string;
            __Unused57: string;
            __Unused58: string;
            __Unused59: string;
            Hrmp: string;
            __Unused61: string;
            ParasDisputes: string;
            ParasSlashing: string;
            OnDemand: string;
            CoretimeAssignmentProvider: string;
            __Unused66: string;
            __Unused67: string;
            __Unused68: string;
            __Unused69: string;
            Registrar: string;
            Slots: string;
            Auctions: string;
            Crowdloan: string;
            Coretime: string;
            __Unused75: string;
            __Unused76: string;
            __Unused77: string;
            __Unused78: string;
            __Unused79: string;
            __Unused80: string;
            __Unused81: string;
            __Unused82: string;
            __Unused83: string;
            __Unused84: string;
            __Unused85: string;
            __Unused86: string;
            __Unused87: string;
            __Unused88: string;
            __Unused89: string;
            __Unused90: string;
            __Unused91: string;
            __Unused92: string;
            __Unused93: string;
            __Unused94: string;
            __Unused95: string;
            __Unused96: string;
            __Unused97: string;
            StateTrieMigration: string;
            XcmPallet: string;
            MessageQueue: string;
            AssetRate: string;
            __Unused102: string;
            __Unused103: string;
            __Unused104: string;
            __Unused105: string;
            __Unused106: string;
            __Unused107: string;
            __Unused108: string;
            __Unused109: string;
            __Unused110: string;
            __Unused111: string;
            __Unused112: string;
            __Unused113: string;
            __Unused114: string;
            __Unused115: string;
            __Unused116: string;
            __Unused117: string;
            __Unused118: string;
            __Unused119: string;
            __Unused120: string;
            __Unused121: string;
            __Unused122: string;
            __Unused123: string;
            __Unused124: string;
            __Unused125: string;
            __Unused126: string;
            __Unused127: string;
            __Unused128: string;
            __Unused129: string;
            __Unused130: string;
            __Unused131: string;
            __Unused132: string;
            __Unused133: string;
            __Unused134: string;
            __Unused135: string;
            __Unused136: string;
            __Unused137: string;
            __Unused138: string;
            __Unused139: string;
            __Unused140: string;
            __Unused141: string;
            __Unused142: string;
            __Unused143: string;
            __Unused144: string;
            __Unused145: string;
            __Unused146: string;
            __Unused147: string;
            __Unused148: string;
            __Unused149: string;
            __Unused150: string;
            __Unused151: string;
            __Unused152: string;
            __Unused153: string;
            __Unused154: string;
            __Unused155: string;
            __Unused156: string;
            __Unused157: string;
            __Unused158: string;
            __Unused159: string;
            __Unused160: string;
            __Unused161: string;
            __Unused162: string;
            __Unused163: string;
            __Unused164: string;
            __Unused165: string;
            __Unused166: string;
            __Unused167: string;
            __Unused168: string;
            __Unused169: string;
            __Unused170: string;
            __Unused171: string;
            __Unused172: string;
            __Unused173: string;
            __Unused174: string;
            __Unused175: string;
            __Unused176: string;
            __Unused177: string;
            __Unused178: string;
            __Unused179: string;
            __Unused180: string;
            __Unused181: string;
            __Unused182: string;
            __Unused183: string;
            __Unused184: string;
            __Unused185: string;
            __Unused186: string;
            __Unused187: string;
            __Unused188: string;
            __Unused189: string;
            __Unused190: string;
            __Unused191: string;
            __Unused192: string;
            __Unused193: string;
            __Unused194: string;
            __Unused195: string;
            __Unused196: string;
            __Unused197: string;
            __Unused198: string;
            __Unused199: string;
            Beefy: string;
        };
    };
};
export default _default;
