declare const _default: {
    /**
     * Lookup3: frame_system::AccountInfo<Nonce, pallet_balances::types::AccountData<Balance>>
     **/
    FrameSystemAccountInfo: {
        nonce: string;
        consumers: string;
        providers: string;
        sufficients: string;
        data: string;
    };
    /**
     * Lookup5: pallet_balances::types::AccountData<Balance>
     **/
    PalletBalancesAccountData: {
        free: string;
        reserved: string;
        frozen: string;
        flags: string;
    };
    /**
     * Lookup9: frame_support::dispatch::PerDispatchClass<sp_weights::weight_v2::Weight>
     **/
    FrameSupportDispatchPerDispatchClassWeight: {
        normal: string;
        operational: string;
        mandatory: string;
    };
    /**
     * Lookup10: sp_weights::weight_v2::Weight
     **/
    SpWeightsWeightV2Weight: {
        refTime: string;
        proofSize: string;
    };
    /**
     * Lookup15: sp_runtime::generic::digest::Digest
     **/
    SpRuntimeDigest: {
        logs: string;
    };
    /**
     * Lookup17: sp_runtime::generic::digest::DigestItem
     **/
    SpRuntimeDigestDigestItem: {
        _enum: {
            Other: string;
            __Unused1: string;
            __Unused2: string;
            __Unused3: string;
            Consensus: string;
            Seal: string;
            PreRuntime: string;
            __Unused7: string;
            RuntimeEnvironmentUpdated: string;
        };
    };
    /**
     * Lookup20: frame_system::EventRecord<kitchensink_runtime::RuntimeEvent, primitive_types::H256>
     **/
    FrameSystemEventRecord: {
        phase: string;
        event: string;
        topics: string;
    };
    /**
     * Lookup22: frame_system::pallet::Event<T>
     **/
    FrameSystemEvent: {
        _enum: {
            ExtrinsicSuccess: {
                dispatchInfo: string;
            };
            ExtrinsicFailed: {
                dispatchError: string;
                dispatchInfo: string;
            };
            CodeUpdated: string;
            NewAccount: {
                account: string;
            };
            KilledAccount: {
                account: string;
            };
            Remarked: {
                _alias: {
                    hash_: string;
                };
                sender: string;
                hash_: string;
            };
            UpgradeAuthorized: {
                codeHash: string;
                checkVersion: string;
            };
            RejectedInvalidAuthorizedUpgrade: {
                codeHash: string;
                error: string;
            };
        };
    };
    /**
     * Lookup23: frame_system::DispatchEventInfo
     **/
    FrameSystemDispatchEventInfo: {
        weight: string;
        class: string;
        paysFee: string;
    };
    /**
     * Lookup24: frame_support::dispatch::DispatchClass
     **/
    FrameSupportDispatchDispatchClass: {
        _enum: string[];
    };
    /**
     * Lookup25: frame_support::dispatch::Pays
     **/
    FrameSupportDispatchPays: {
        _enum: string[];
    };
    /**
     * Lookup26: sp_runtime::DispatchError
     **/
    SpRuntimeDispatchError: {
        _enum: {
            Other: string;
            CannotLookup: string;
            BadOrigin: string;
            Module: string;
            ConsumerRemaining: string;
            NoProviders: string;
            TooManyConsumers: string;
            Token: string;
            Arithmetic: string;
            Transactional: string;
            Exhausted: string;
            Corruption: string;
            Unavailable: string;
            RootNotAllowed: string;
            Trie: string;
        };
    };
    /**
     * Lookup27: sp_runtime::ModuleError
     **/
    SpRuntimeModuleError: {
        index: string;
        error: string;
    };
    /**
     * Lookup28: sp_runtime::TokenError
     **/
    SpRuntimeTokenError: {
        _enum: string[];
    };
    /**
     * Lookup29: sp_arithmetic::ArithmeticError
     **/
    SpArithmeticArithmeticError: {
        _enum: string[];
    };
    /**
     * Lookup30: sp_runtime::TransactionalError
     **/
    SpRuntimeTransactionalError: {
        _enum: string[];
    };
    /**
     * Lookup31: sp_runtime::proving_trie::TrieError
     **/
    SpRuntimeProvingTrieTrieError: {
        _enum: string[];
    };
    /**
     * Lookup32: pallet_utility::pallet::Event
     **/
    PalletUtilityEvent: {
        _enum: {
            BatchInterrupted: {
                index: string;
                error: string;
            };
            BatchCompleted: string;
            BatchCompletedWithErrors: string;
            ItemCompleted: string;
            ItemFailed: {
                error: string;
            };
            DispatchedAs: {
                result: string;
            };
            IfElseMainSuccess: string;
            IfElseFallbackCalled: {
                mainError: string;
            };
        };
    };
    /**
     * Lookup35: pallet_indices::pallet::Event<T>
     **/
    PalletIndicesEvent: {
        _enum: {
            IndexAssigned: {
                who: string;
                index: string;
            };
            IndexFreed: {
                index: string;
            };
            IndexFrozen: {
                index: string;
                who: string;
            };
            DepositPoked: {
                who: string;
                index: string;
                oldDeposit: string;
                newDeposit: string;
            };
        };
    };
    /**
     * Lookup36: pallet_balances::pallet::Event<T, I>
     **/
    PalletBalancesEvent: {
        _enum: {
            Endowed: {
                account: string;
                freeBalance: string;
            };
            DustLost: {
                account: string;
                amount: string;
            };
            Transfer: {
                from: string;
                to: string;
                amount: string;
            };
            BalanceSet: {
                who: string;
                free: string;
            };
            Reserved: {
                who: string;
                amount: string;
            };
            Unreserved: {
                who: string;
                amount: string;
            };
            ReserveRepatriated: {
                from: string;
                to: string;
                amount: string;
                destinationStatus: string;
            };
            Deposit: {
                who: string;
                amount: string;
            };
            Withdraw: {
                who: string;
                amount: string;
            };
            Slashed: {
                who: string;
                amount: string;
            };
            Minted: {
                who: string;
                amount: string;
            };
            Burned: {
                who: string;
                amount: string;
            };
            Suspended: {
                who: string;
                amount: string;
            };
            Restored: {
                who: string;
                amount: string;
            };
            Upgraded: {
                who: string;
            };
            Issued: {
                amount: string;
            };
            Rescinded: {
                amount: string;
            };
            Locked: {
                who: string;
                amount: string;
            };
            Unlocked: {
                who: string;
                amount: string;
            };
            Frozen: {
                who: string;
                amount: string;
            };
            Thawed: {
                who: string;
                amount: string;
            };
            TotalIssuanceForced: {
                _alias: {
                    new_: string;
                };
                old: string;
                new_: string;
            };
        };
    };
    /**
     * Lookup37: frame_support::traits::tokens::misc::BalanceStatus
     **/
    FrameSupportTokensMiscBalanceStatus: {
        _enum: string[];
    };
    /**
     * Lookup38: pallet_transaction_payment::pallet::Event<T>
     **/
    PalletTransactionPaymentEvent: {
        _enum: {
            TransactionFeePaid: {
                who: string;
                actualFee: string;
                tip: string;
            };
        };
    };
    /**
     * Lookup39: pallet_asset_conversion_tx_payment::pallet::Event<T>
     **/
    PalletAssetConversionTxPaymentEvent: {
        _enum: {
            AssetTxFeePaid: {
                who: string;
                actualFee: string;
                tip: string;
                assetId: string;
            };
            AssetRefundFailed: {
                nativeAmountKept: string;
            };
        };
    };
    /**
     * Lookup40: frame_support::traits::tokens::fungible::union_of::NativeOrWithId<AssetId>
     **/
    FrameSupportTokensFungibleUnionOfNativeOrWithId: {
        _enum: {
            Native: string;
            WithId: string;
        };
    };
    /**
     * Lookup41: pallet_election_provider_multi_phase::pallet::Event<T>
     **/
    PalletElectionProviderMultiPhaseEvent: {
        _enum: {
            SolutionStored: {
                compute: string;
                origin: string;
                prevEjected: string;
            };
            ElectionFinalized: {
                compute: string;
                score: string;
            };
            ElectionFailed: string;
            Rewarded: {
                account: string;
                value: string;
            };
            Slashed: {
                account: string;
                value: string;
            };
            PhaseTransitioned: {
                from: string;
                to: string;
                round: string;
            };
        };
    };
    /**
     * Lookup42: pallet_election_provider_multi_phase::ElectionCompute
     **/
    PalletElectionProviderMultiPhaseElectionCompute: {
        _enum: string[];
    };
    /**
     * Lookup44: sp_npos_elections::ElectionScore
     **/
    SpNposElectionsElectionScore: {
        minimalStake: string;
        sumStake: string;
        sumStakeSquared: string;
    };
    /**
     * Lookup45: pallet_election_provider_multi_phase::Phase<Bn>
     **/
    PalletElectionProviderMultiPhasePhase: {
        _enum: {
            Off: string;
            Signed: string;
            Unsigned: string;
            Emergency: string;
        };
    };
    /**
     * Lookup47: pallet_staking::pallet::pallet::Event<T>
     **/
    PalletStakingPalletEvent: {
        _enum: {
            EraPaid: {
                eraIndex: string;
                validatorPayout: string;
                remainder: string;
            };
            Rewarded: {
                stash: string;
                dest: string;
                amount: string;
            };
            Slashed: {
                staker: string;
                amount: string;
            };
            SlashReported: {
                validator: string;
                fraction: string;
                slashEra: string;
            };
            OldSlashingReportDiscarded: {
                sessionIndex: string;
            };
            StakersElected: string;
            Bonded: {
                stash: string;
                amount: string;
            };
            Unbonded: {
                stash: string;
                amount: string;
            };
            Withdrawn: {
                stash: string;
                amount: string;
            };
            Kicked: {
                nominator: string;
                stash: string;
            };
            StakingElectionFailed: string;
            Chilled: {
                stash: string;
            };
            PayoutStarted: {
                eraIndex: string;
                validatorStash: string;
                page: string;
                next: string;
            };
            ValidatorPrefsSet: {
                stash: string;
                prefs: string;
            };
            SnapshotVotersSizeExceeded: {
                _alias: {
                    size_: string;
                };
                size_: string;
            };
            SnapshotTargetsSizeExceeded: {
                _alias: {
                    size_: string;
                };
                size_: string;
            };
            ForceEra: {
                mode: string;
            };
            ControllerBatchDeprecated: {
                failures: string;
            };
            CurrencyMigrated: {
                stash: string;
                forceWithdraw: string;
            };
        };
    };
    /**
     * Lookup48: pallet_staking::RewardDestination<sp_core::crypto::AccountId32>
     **/
    PalletStakingRewardDestination: {
        _enum: {
            Staked: string;
            Stash: string;
            Controller: string;
            Account: string;
            None: string;
        };
    };
    /**
     * Lookup51: pallet_staking::ValidatorPrefs
     **/
    PalletStakingValidatorPrefs: {
        commission: string;
        blocked: string;
    };
    /**
     * Lookup53: pallet_staking::Forcing
     **/
    PalletStakingForcing: {
        _enum: string[];
    };
    /**
     * Lookup54: pallet_session::pallet::Event<T>
     **/
    PalletSessionEvent: {
        _enum: {
            NewSession: {
                sessionIndex: string;
            };
            ValidatorDisabled: {
                validator: string;
            };
            ValidatorReenabled: {
                validator: string;
            };
        };
    };
    /**
     * Lookup55: pallet_democracy::pallet::Event<T>
     **/
    PalletDemocracyEvent: {
        _enum: {
            Proposed: {
                proposalIndex: string;
                deposit: string;
            };
            Tabled: {
                proposalIndex: string;
                deposit: string;
            };
            ExternalTabled: string;
            Started: {
                refIndex: string;
                threshold: string;
            };
            Passed: {
                refIndex: string;
            };
            NotPassed: {
                refIndex: string;
            };
            Cancelled: {
                refIndex: string;
            };
            Delegated: {
                who: string;
                target: string;
            };
            Undelegated: {
                account: string;
            };
            Vetoed: {
                who: string;
                proposalHash: string;
                until: string;
            };
            Blacklisted: {
                proposalHash: string;
            };
            Voted: {
                voter: string;
                refIndex: string;
                vote: string;
            };
            Seconded: {
                seconder: string;
                propIndex: string;
            };
            ProposalCanceled: {
                propIndex: string;
            };
            MetadataSet: {
                _alias: {
                    hash_: string;
                };
                owner: string;
                hash_: string;
            };
            MetadataCleared: {
                _alias: {
                    hash_: string;
                };
                owner: string;
                hash_: string;
            };
            MetadataTransferred: {
                _alias: {
                    hash_: string;
                };
                prevOwner: string;
                owner: string;
                hash_: string;
            };
        };
    };
    /**
     * Lookup56: pallet_democracy::vote_threshold::VoteThreshold
     **/
    PalletDemocracyVoteThreshold: {
        _enum: string[];
    };
    /**
     * Lookup57: pallet_democracy::vote::AccountVote<Balance>
     **/
    PalletDemocracyVoteAccountVote: {
        _enum: {
            Standard: {
                vote: string;
                balance: string;
            };
            Split: {
                aye: string;
                nay: string;
            };
        };
    };
    /**
     * Lookup59: pallet_democracy::types::MetadataOwner
     **/
    PalletDemocracyMetadataOwner: {
        _enum: {
            External: string;
            Proposal: string;
            Referendum: string;
        };
    };
    /**
     * Lookup60: pallet_collective::pallet::Event<T, I>
     **/
    PalletCollectiveEvent: {
        _enum: {
            Proposed: {
                account: string;
                proposalIndex: string;
                proposalHash: string;
                threshold: string;
            };
            Voted: {
                account: string;
                proposalHash: string;
                voted: string;
                yes: string;
                no: string;
            };
            Approved: {
                proposalHash: string;
            };
            Disapproved: {
                proposalHash: string;
            };
            Executed: {
                proposalHash: string;
                result: string;
            };
            MemberExecuted: {
                proposalHash: string;
                result: string;
            };
            Closed: {
                proposalHash: string;
                yes: string;
                no: string;
            };
            Killed: {
                proposalHash: string;
            };
            ProposalCostBurned: {
                proposalHash: string;
                who: string;
            };
            ProposalCostReleased: {
                proposalHash: string;
                who: string;
            };
        };
    };
    /**
     * Lookup62: pallet_elections_phragmen::pallet::Event<T>
     **/
    PalletElectionsPhragmenEvent: {
        _enum: {
            NewTerm: {
                newMembers: string;
            };
            EmptyTerm: string;
            ElectionError: string;
            MemberKicked: {
                member: string;
            };
            Renounced: {
                candidate: string;
            };
            CandidateSlashed: {
                candidate: string;
                amount: string;
            };
            SeatHolderSlashed: {
                seatHolder: string;
                amount: string;
            };
        };
    };
    /**
     * Lookup65: pallet_membership::pallet::Event<T, I>
     **/
    PalletMembershipEvent: {
        _enum: string[];
    };
    /**
     * Lookup66: pallet_grandpa::pallet::Event
     **/
    PalletGrandpaEvent: {
        _enum: {
            NewAuthorities: {
                authoritySet: string;
            };
            Paused: string;
            Resumed: string;
        };
    };
    /**
     * Lookup69: sp_consensus_grandpa::app::Public
     **/
    SpConsensusGrandpaAppPublic: string;
    /**
     * Lookup70: pallet_treasury::pallet::Event<T, I>
     **/
    PalletTreasuryEvent: {
        _enum: {
            Spending: {
                budgetRemaining: string;
            };
            Awarded: {
                proposalIndex: string;
                award: string;
                account: string;
            };
            Burnt: {
                burntFunds: string;
            };
            Rollover: {
                rolloverBalance: string;
            };
            Deposit: {
                value: string;
            };
            SpendApproved: {
                proposalIndex: string;
                amount: string;
                beneficiary: string;
            };
            UpdatedInactive: {
                reactivated: string;
                deactivated: string;
            };
            AssetSpendApproved: {
                index: string;
                assetKind: string;
                amount: string;
                beneficiary: string;
                validFrom: string;
                expireAt: string;
            };
            AssetSpendVoided: {
                index: string;
            };
            Paid: {
                index: string;
                paymentId: string;
            };
            PaymentFailed: {
                index: string;
                paymentId: string;
            };
            SpendProcessed: {
                index: string;
            };
        };
    };
    /**
     * Lookup71: pallet_asset_rate::pallet::Event<T>
     **/
    PalletAssetRateEvent: {
        _enum: {
            AssetRateCreated: {
                assetKind: string;
                rate: string;
            };
            AssetRateRemoved: {
                assetKind: string;
            };
            AssetRateUpdated: {
                _alias: {
                    new_: string;
                };
                assetKind: string;
                old: string;
                new_: string;
            };
        };
    };
    /**
     * Lookup73: pallet_contracts::pallet::Event<T>
     **/
    PalletContractsEvent: {
        _enum: {
            Instantiated: {
                deployer: string;
                contract: string;
            };
            Terminated: {
                contract: string;
                beneficiary: string;
            };
            CodeStored: {
                codeHash: string;
                depositHeld: string;
                uploader: string;
            };
            ContractEmitted: {
                contract: string;
                data: string;
            };
            CodeRemoved: {
                codeHash: string;
                depositReleased: string;
                remover: string;
            };
            ContractCodeUpdated: {
                contract: string;
                newCodeHash: string;
                oldCodeHash: string;
            };
            Called: {
                caller: string;
                contract: string;
            };
            DelegateCalled: {
                contract: string;
                codeHash: string;
            };
            StorageDepositTransferredAndHeld: {
                from: string;
                to: string;
                amount: string;
            };
            StorageDepositTransferredAndReleased: {
                from: string;
                to: string;
                amount: string;
            };
        };
    };
    /**
     * Lookup74: pallet_contracts::Origin<kitchensink_runtime::Runtime>
     **/
    PalletContractsOrigin: {
        _enum: {
            Root: string;
            Signed: string;
        };
    };
    /**
     * Lookup75: kitchensink_runtime::Runtime
     **/
    KitchensinkRuntimeRuntime: string;
    /**
     * Lookup76: pallet_sudo::pallet::Event<T>
     **/
    PalletSudoEvent: {
        _enum: {
            Sudid: {
                sudoResult: string;
            };
            KeyChanged: {
                _alias: {
                    new_: string;
                };
                old: string;
                new_: string;
            };
            KeyRemoved: string;
            SudoAsDone: {
                sudoResult: string;
            };
        };
    };
    /**
     * Lookup77: pallet_im_online::pallet::Event<T>
     **/
    PalletImOnlineEvent: {
        _enum: {
            HeartbeatReceived: {
                authorityId: string;
            };
            AllGood: string;
            SomeOffline: {
                offline: string;
            };
        };
    };
    /**
     * Lookup78: pallet_im_online::sr25519::app_sr25519::Public
     **/
    PalletImOnlineSr25519AppSr25519Public: string;
    /**
     * Lookup81: pallet_offences::pallet::Event
     **/
    PalletOffencesEvent: {
        _enum: {
            Offence: {
                kind: string;
                timeslot: string;
            };
        };
    };
    /**
     * Lookup83: pallet_identity::pallet::Event<T>
     **/
    PalletIdentityEvent: {
        _enum: {
            IdentitySet: {
                who: string;
            };
            IdentityCleared: {
                who: string;
                deposit: string;
            };
            IdentityKilled: {
                who: string;
                deposit: string;
            };
            JudgementRequested: {
                who: string;
                registrarIndex: string;
            };
            JudgementUnrequested: {
                who: string;
                registrarIndex: string;
            };
            JudgementGiven: {
                target: string;
                registrarIndex: string;
            };
            RegistrarAdded: {
                registrarIndex: string;
            };
            SubIdentityAdded: {
                sub: string;
                main: string;
                deposit: string;
            };
            SubIdentitiesSet: {
                main: string;
                numberOfSubs: string;
                newDeposit: string;
            };
            SubIdentityRenamed: {
                sub: string;
                main: string;
            };
            SubIdentityRemoved: {
                sub: string;
                main: string;
                deposit: string;
            };
            SubIdentityRevoked: {
                sub: string;
                main: string;
                deposit: string;
            };
            AuthorityAdded: {
                authority: string;
            };
            AuthorityRemoved: {
                authority: string;
            };
            UsernameSet: {
                who: string;
                username: string;
            };
            UsernameQueued: {
                who: string;
                username: string;
                expiration: string;
            };
            PreapprovalExpired: {
                whose: string;
            };
            PrimaryUsernameSet: {
                who: string;
                username: string;
            };
            DanglingUsernameRemoved: {
                who: string;
                username: string;
            };
            UsernameUnbound: {
                username: string;
            };
            UsernameRemoved: {
                username: string;
            };
            UsernameKilled: {
                username: string;
            };
        };
    };
    /**
     * Lookup85: pallet_society::pallet::Event<T, I>
     **/
    PalletSocietyEvent: {
        _enum: {
            Founded: {
                founder: string;
            };
            Bid: {
                candidateId: string;
                offer: string;
            };
            Vouch: {
                candidateId: string;
                offer: string;
                vouching: string;
            };
            AutoUnbid: {
                candidate: string;
            };
            Unbid: {
                candidate: string;
            };
            Unvouch: {
                candidate: string;
            };
            Inducted: {
                primary: string;
                candidates: string;
            };
            SuspendedMemberJudgement: {
                who: string;
                judged: string;
            };
            CandidateSuspended: {
                candidate: string;
            };
            MemberSuspended: {
                member: string;
            };
            Challenged: {
                member: string;
            };
            Vote: {
                candidate: string;
                voter: string;
                vote: string;
            };
            DefenderVote: {
                voter: string;
                vote: string;
            };
            NewParams: {
                params: string;
            };
            Unfounded: {
                founder: string;
            };
            Deposit: {
                value: string;
            };
            Elevated: {
                member: string;
                rank: string;
            };
        };
    };
    /**
     * Lookup87: pallet_society::GroupParams<Balance>
     **/
    PalletSocietyGroupParams: {
        maxMembers: string;
        maxIntake: string;
        maxStrikes: string;
        candidateDeposit: string;
    };
    /**
     * Lookup88: pallet_recovery::pallet::Event<T>
     **/
    PalletRecoveryEvent: {
        _enum: {
            RecoveryCreated: {
                account: string;
            };
            RecoveryInitiated: {
                lostAccount: string;
                rescuerAccount: string;
            };
            RecoveryVouched: {
                lostAccount: string;
                rescuerAccount: string;
                sender: string;
            };
            RecoveryClosed: {
                lostAccount: string;
                rescuerAccount: string;
            };
            AccountRecovered: {
                lostAccount: string;
                rescuerAccount: string;
            };
            RecoveryRemoved: {
                lostAccount: string;
            };
        };
    };
    /**
     * Lookup89: pallet_vesting::pallet::Event<T>
     **/
    PalletVestingEvent: {
        _enum: {
            VestingUpdated: {
                account: string;
                unvested: string;
            };
            VestingCompleted: {
                account: string;
            };
        };
    };
    /**
     * Lookup90: pallet_scheduler::pallet::Event<T>
     **/
    PalletSchedulerEvent: {
        _enum: {
            Scheduled: {
                when: string;
                index: string;
            };
            Canceled: {
                when: string;
                index: string;
            };
            Dispatched: {
                task: string;
                id: string;
                result: string;
            };
            RetrySet: {
                task: string;
                id: string;
                period: string;
                retries: string;
            };
            RetryCancelled: {
                task: string;
                id: string;
            };
            CallUnavailable: {
                task: string;
                id: string;
            };
            PeriodicFailed: {
                task: string;
                id: string;
            };
            RetryFailed: {
                task: string;
                id: string;
            };
            PermanentlyOverweight: {
                task: string;
                id: string;
            };
            AgendaIncomplete: {
                when: string;
            };
        };
    };
    /**
     * Lookup93: pallet_glutton::pallet::Event
     **/
    PalletGluttonEvent: {
        _enum: {
            PalletInitialized: {
                reinit: string;
            };
            ComputationLimitSet: {
                compute: string;
            };
            StorageLimitSet: {
                storage: string;
            };
            BlockLengthLimitSet: {
                blockLength: string;
            };
        };
    };
    /**
     * Lookup95: pallet_preimage::pallet::Event<T>
     **/
    PalletPreimageEvent: {
        _enum: {
            Noted: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            Requested: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            Cleared: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
        };
    };
    /**
     * Lookup96: pallet_proxy::pallet::Event<T>
     **/
    PalletProxyEvent: {
        _enum: {
            ProxyExecuted: {
                result: string;
            };
            PureCreated: {
                pure: string;
                who: string;
                proxyType: string;
                disambiguationIndex: string;
            };
            Announced: {
                real: string;
                proxy: string;
                callHash: string;
            };
            ProxyAdded: {
                delegator: string;
                delegatee: string;
                proxyType: string;
                delay: string;
            };
            ProxyRemoved: {
                delegator: string;
                delegatee: string;
                proxyType: string;
                delay: string;
            };
            DepositPoked: {
                who: string;
                kind: string;
                oldDeposit: string;
                newDeposit: string;
            };
        };
    };
    /**
     * Lookup97: kitchensink_runtime::ProxyType
     **/
    KitchensinkRuntimeProxyType: {
        _enum: string[];
    };
    /**
     * Lookup99: pallet_proxy::DepositKind
     **/
    PalletProxyDepositKind: {
        _enum: string[];
    };
    /**
     * Lookup100: pallet_multisig::pallet::Event<T>
     **/
    PalletMultisigEvent: {
        _enum: {
            NewMultisig: {
                approving: string;
                multisig: string;
                callHash: string;
            };
            MultisigApproval: {
                approving: string;
                timepoint: string;
                multisig: string;
                callHash: string;
            };
            MultisigExecuted: {
                approving: string;
                timepoint: string;
                multisig: string;
                callHash: string;
                result: string;
            };
            MultisigCancelled: {
                cancelling: string;
                timepoint: string;
                multisig: string;
                callHash: string;
            };
            DepositPoked: {
                who: string;
                callHash: string;
                oldDeposit: string;
                newDeposit: string;
            };
        };
    };
    /**
     * Lookup101: pallet_multisig::Timepoint<BlockNumber>
     **/
    PalletMultisigTimepoint: {
        height: string;
        index: string;
    };
    /**
     * Lookup102: pallet_bounties::pallet::Event<T, I>
     **/
    PalletBountiesEvent: {
        _enum: {
            BountyProposed: {
                index: string;
            };
            BountyRejected: {
                index: string;
                bond: string;
            };
            BountyBecameActive: {
                index: string;
            };
            BountyAwarded: {
                index: string;
                beneficiary: string;
            };
            BountyClaimed: {
                index: string;
                payout: string;
                beneficiary: string;
            };
            BountyCanceled: {
                index: string;
            };
            BountyExtended: {
                index: string;
            };
            BountyApproved: {
                index: string;
            };
            CuratorProposed: {
                bountyId: string;
                curator: string;
            };
            CuratorUnassigned: {
                bountyId: string;
            };
            CuratorAccepted: {
                bountyId: string;
                curator: string;
            };
        };
    };
    /**
     * Lookup103: pallet_tips::pallet::Event<T, I>
     **/
    PalletTipsEvent: {
        _enum: {
            NewTip: {
                tipHash: string;
            };
            TipClosing: {
                tipHash: string;
            };
            TipClosed: {
                tipHash: string;
                who: string;
                payout: string;
            };
            TipRetracted: {
                tipHash: string;
            };
            TipSlashed: {
                tipHash: string;
                finder: string;
                deposit: string;
            };
        };
    };
    /**
     * Lookup104: pallet_assets::pallet::Event<T, I>
     **/
    PalletAssetsEvent: {
        _enum: {
            Created: {
                assetId: string;
                creator: string;
                owner: string;
            };
            Issued: {
                assetId: string;
                owner: string;
                amount: string;
            };
            Transferred: {
                assetId: string;
                from: string;
                to: string;
                amount: string;
            };
            Burned: {
                assetId: string;
                owner: string;
                balance: string;
            };
            TeamChanged: {
                assetId: string;
                issuer: string;
                admin: string;
                freezer: string;
            };
            OwnerChanged: {
                assetId: string;
                owner: string;
            };
            Frozen: {
                assetId: string;
                who: string;
            };
            Thawed: {
                assetId: string;
                who: string;
            };
            AssetFrozen: {
                assetId: string;
            };
            AssetThawed: {
                assetId: string;
            };
            AccountsDestroyed: {
                assetId: string;
                accountsDestroyed: string;
                accountsRemaining: string;
            };
            ApprovalsDestroyed: {
                assetId: string;
                approvalsDestroyed: string;
                approvalsRemaining: string;
            };
            DestructionStarted: {
                assetId: string;
            };
            Destroyed: {
                assetId: string;
            };
            ForceCreated: {
                assetId: string;
                owner: string;
            };
            MetadataSet: {
                assetId: string;
                name: string;
                symbol: string;
                decimals: string;
                isFrozen: string;
            };
            MetadataCleared: {
                assetId: string;
            };
            ApprovedTransfer: {
                assetId: string;
                source: string;
                delegate: string;
                amount: string;
            };
            ApprovalCancelled: {
                assetId: string;
                owner: string;
                delegate: string;
            };
            TransferredApproved: {
                assetId: string;
                owner: string;
                delegate: string;
                destination: string;
                amount: string;
            };
            AssetStatusChanged: {
                assetId: string;
            };
            AssetMinBalanceChanged: {
                assetId: string;
                newMinBalance: string;
            };
            Touched: {
                assetId: string;
                who: string;
                depositor: string;
            };
            Blocked: {
                assetId: string;
                who: string;
            };
            Deposited: {
                assetId: string;
                who: string;
                amount: string;
            };
            Withdrawn: {
                assetId: string;
                who: string;
                amount: string;
            };
        };
    };
    /**
     * Lookup106: pallet_lottery::pallet::Event<T>
     **/
    PalletLotteryEvent: {
        _enum: {
            LotteryStarted: string;
            CallsUpdated: string;
            Winner: {
                winner: string;
                lotteryBalance: string;
            };
            TicketBought: {
                who: string;
                callIndex: string;
            };
        };
    };
    /**
     * Lookup108: pallet_nis::pallet::Event<T>
     **/
    PalletNisEvent: {
        _enum: {
            BidPlaced: {
                who: string;
                amount: string;
                duration: string;
            };
            BidRetracted: {
                who: string;
                amount: string;
                duration: string;
            };
            BidDropped: {
                who: string;
                amount: string;
                duration: string;
            };
            Issued: {
                index: string;
                expiry: string;
                who: string;
                proportion: string;
                amount: string;
            };
            Thawed: {
                index: string;
                who: string;
                proportion: string;
                amount: string;
                dropped: string;
            };
            Funded: {
                deficit: string;
            };
            Transferred: {
                from: string;
                to: string;
                index: string;
            };
        };
    };
    /**
     * Lookup110: pallet_uniques::pallet::Event<T, I>
     **/
    PalletUniquesEvent: {
        _enum: {
            Created: {
                collection: string;
                creator: string;
                owner: string;
            };
            ForceCreated: {
                collection: string;
                owner: string;
            };
            Destroyed: {
                collection: string;
            };
            Issued: {
                collection: string;
                item: string;
                owner: string;
            };
            Transferred: {
                collection: string;
                item: string;
                from: string;
                to: string;
            };
            Burned: {
                collection: string;
                item: string;
                owner: string;
            };
            Frozen: {
                collection: string;
                item: string;
            };
            Thawed: {
                collection: string;
                item: string;
            };
            CollectionFrozen: {
                collection: string;
            };
            CollectionThawed: {
                collection: string;
            };
            OwnerChanged: {
                collection: string;
                newOwner: string;
            };
            TeamChanged: {
                collection: string;
                issuer: string;
                admin: string;
                freezer: string;
            };
            ApprovedTransfer: {
                collection: string;
                item: string;
                owner: string;
                delegate: string;
            };
            ApprovalCancelled: {
                collection: string;
                item: string;
                owner: string;
                delegate: string;
            };
            ItemStatusChanged: {
                collection: string;
            };
            CollectionMetadataSet: {
                collection: string;
                data: string;
                isFrozen: string;
            };
            CollectionMetadataCleared: {
                collection: string;
            };
            MetadataSet: {
                collection: string;
                item: string;
                data: string;
                isFrozen: string;
            };
            MetadataCleared: {
                collection: string;
                item: string;
            };
            Redeposited: {
                collection: string;
                successfulItems: string;
            };
            AttributeSet: {
                collection: string;
                maybeItem: string;
                key: string;
                value: string;
            };
            AttributeCleared: {
                collection: string;
                maybeItem: string;
                key: string;
            };
            OwnershipAcceptanceChanged: {
                who: string;
                maybeCollection: string;
            };
            CollectionMaxSupplySet: {
                collection: string;
                maxSupply: string;
            };
            ItemPriceSet: {
                collection: string;
                item: string;
                price: string;
                whitelistedBuyer: string;
            };
            ItemPriceRemoved: {
                collection: string;
                item: string;
            };
            ItemBought: {
                collection: string;
                item: string;
                price: string;
                seller: string;
                buyer: string;
            };
        };
    };
    /**
     * Lookup114: pallet_nfts::pallet::Event<T, I>
     **/
    PalletNftsEvent: {
        _enum: {
            Created: {
                collection: string;
                creator: string;
                owner: string;
            };
            ForceCreated: {
                collection: string;
                owner: string;
            };
            Destroyed: {
                collection: string;
            };
            Issued: {
                collection: string;
                item: string;
                owner: string;
            };
            Transferred: {
                collection: string;
                item: string;
                from: string;
                to: string;
            };
            Burned: {
                collection: string;
                item: string;
                owner: string;
            };
            ItemTransferLocked: {
                collection: string;
                item: string;
            };
            ItemTransferUnlocked: {
                collection: string;
                item: string;
            };
            ItemPropertiesLocked: {
                collection: string;
                item: string;
                lockMetadata: string;
                lockAttributes: string;
            };
            CollectionLocked: {
                collection: string;
            };
            OwnerChanged: {
                collection: string;
                newOwner: string;
            };
            TeamChanged: {
                collection: string;
                issuer: string;
                admin: string;
                freezer: string;
            };
            TransferApproved: {
                collection: string;
                item: string;
                owner: string;
                delegate: string;
                deadline: string;
            };
            ApprovalCancelled: {
                collection: string;
                item: string;
                owner: string;
                delegate: string;
            };
            AllApprovalsCancelled: {
                collection: string;
                item: string;
                owner: string;
            };
            CollectionConfigChanged: {
                collection: string;
            };
            CollectionMetadataSet: {
                collection: string;
                data: string;
            };
            CollectionMetadataCleared: {
                collection: string;
            };
            ItemMetadataSet: {
                collection: string;
                item: string;
                data: string;
            };
            ItemMetadataCleared: {
                collection: string;
                item: string;
            };
            Redeposited: {
                collection: string;
                successfulItems: string;
            };
            AttributeSet: {
                collection: string;
                maybeItem: string;
                key: string;
                value: string;
                namespace: string;
            };
            AttributeCleared: {
                collection: string;
                maybeItem: string;
                key: string;
                namespace: string;
            };
            ItemAttributesApprovalAdded: {
                collection: string;
                item: string;
                delegate: string;
            };
            ItemAttributesApprovalRemoved: {
                collection: string;
                item: string;
                delegate: string;
            };
            OwnershipAcceptanceChanged: {
                who: string;
                maybeCollection: string;
            };
            CollectionMaxSupplySet: {
                collection: string;
                maxSupply: string;
            };
            CollectionMintSettingsUpdated: {
                collection: string;
            };
            NextCollectionIdIncremented: {
                nextId: string;
            };
            ItemPriceSet: {
                collection: string;
                item: string;
                price: string;
                whitelistedBuyer: string;
            };
            ItemPriceRemoved: {
                collection: string;
                item: string;
            };
            ItemBought: {
                collection: string;
                item: string;
                price: string;
                seller: string;
                buyer: string;
            };
            TipSent: {
                collection: string;
                item: string;
                sender: string;
                receiver: string;
                amount: string;
            };
            SwapCreated: {
                offeredCollection: string;
                offeredItem: string;
                desiredCollection: string;
                desiredItem: string;
                price: string;
                deadline: string;
            };
            SwapCancelled: {
                offeredCollection: string;
                offeredItem: string;
                desiredCollection: string;
                desiredItem: string;
                price: string;
                deadline: string;
            };
            SwapClaimed: {
                sentCollection: string;
                sentItem: string;
                sentItemOwner: string;
                receivedCollection: string;
                receivedItem: string;
                receivedItemOwner: string;
                price: string;
                deadline: string;
            };
            PreSignedAttributesSet: {
                collection: string;
                item: string;
                namespace: string;
            };
            PalletAttributeSet: {
                collection: string;
                item: string;
                attribute: string;
                value: string;
            };
        };
    };
    /**
     * Lookup116: pallet_nfts::types::AttributeNamespace<sp_core::crypto::AccountId32>
     **/
    PalletNftsAttributeNamespace: {
        _enum: {
            Pallet: string;
            CollectionOwner: string;
            ItemOwner: string;
            Account: string;
        };
    };
    /**
     * Lookup118: pallet_nfts::types::PriceWithDirection<Amount>
     **/
    PalletNftsPriceWithDirection: {
        amount: string;
        direction: string;
    };
    /**
     * Lookup119: pallet_nfts::types::PriceDirection
     **/
    PalletNftsPriceDirection: {
        _enum: string[];
    };
    /**
     * Lookup120: pallet_nfts::types::PalletAttributes<CollectionId>
     **/
    PalletNftsPalletAttributes: {
        _enum: {
            UsedToClaim: string;
            TransferDisabled: string;
        };
    };
    /**
     * Lookup121: pallet_nft_fractionalization::pallet::Event<T>
     **/
    PalletNftFractionalizationEvent: {
        _enum: {
            NftFractionalized: {
                nftCollection: string;
                nft: string;
                fractions: string;
                asset: string;
                beneficiary: string;
            };
            NftUnified: {
                nftCollection: string;
                nft: string;
                asset: string;
                beneficiary: string;
            };
        };
    };
    /**
     * Lookup122: pallet_salary::pallet::Event<T, I>
     **/
    PalletSalaryEvent: {
        _enum: {
            Inducted: {
                who: string;
            };
            Registered: {
                who: string;
                amount: string;
            };
            Paid: {
                who: string;
                beneficiary: string;
                amount: string;
                id: string;
            };
            CycleStarted: {
                index: string;
            };
            Swapped: {
                who: string;
                newWho: string;
            };
        };
    };
    /**
     * Lookup123: pallet_core_fellowship::pallet::Event<T, I>
     **/
    PalletCoreFellowshipEvent: {
        _enum: {
            ParamsChanged: {
                params: string;
            };
            ActiveChanged: {
                who: string;
                isActive: string;
            };
            Inducted: {
                who: string;
            };
            Offboarded: {
                who: string;
            };
            Promoted: {
                who: string;
                toRank: string;
            };
            Demoted: {
                who: string;
                toRank: string;
            };
            Proven: {
                who: string;
                atRank: string;
            };
            Requested: {
                who: string;
                wish: string;
            };
            EvidenceJudged: {
                who: string;
                wish: string;
                evidence: string;
                oldRank: string;
                newRank: string;
            };
            Imported: {
                who: string;
                rank: string;
            };
            Swapped: {
                who: string;
                newWho: string;
            };
        };
    };
    /**
     * Lookup124: pallet_core_fellowship::ParamsType<Balance, BlockNumber, Ranks>
     **/
    PalletCoreFellowshipParamsTypeU128: {
        activeSalary: string;
        passiveSalary: string;
        demotionPeriod: string;
        minPromotionPeriod: string;
        offboardTimeout: string;
    };
    /**
     * Lookup128: pallet_core_fellowship::Wish
     **/
    PalletCoreFellowshipWish: {
        _enum: string[];
    };
    /**
     * Lookup131: pallet_transaction_storage::pallet::Event<T>
     **/
    PalletTransactionStorageEvent: {
        _enum: {
            Stored: {
                index: string;
            };
            Renewed: {
                index: string;
            };
            ProofChecked: string;
        };
    };
    /**
     * Lookup132: pallet_bags_list::pallet::Event<T, I>
     **/
    PalletBagsListEvent: {
        _enum: {
            Rebagged: {
                who: string;
                from: string;
                to: string;
            };
            ScoreUpdated: {
                who: string;
                newScore: string;
            };
        };
    };
    /**
     * Lookup133: pallet_state_trie_migration::pallet::Event<T>
     **/
    PalletStateTrieMigrationEvent: {
        _enum: {
            Migrated: {
                top: string;
                child: string;
                compute: string;
            };
            Slashed: {
                who: string;
                amount: string;
            };
            AutoMigrationFinished: string;
            Halted: {
                error: string;
            };
        };
    };
    /**
     * Lookup134: pallet_state_trie_migration::pallet::MigrationCompute
     **/
    PalletStateTrieMigrationMigrationCompute: {
        _enum: string[];
    };
    /**
     * Lookup135: pallet_state_trie_migration::pallet::Error<T>
     **/
    PalletStateTrieMigrationError: {
        _enum: string[];
    };
    /**
     * Lookup136: pallet_child_bounties::pallet::Event<T>
     **/
    PalletChildBountiesEvent: {
        _enum: {
            Added: {
                index: string;
                childIndex: string;
            };
            Awarded: {
                index: string;
                childIndex: string;
                beneficiary: string;
            };
            Claimed: {
                index: string;
                childIndex: string;
                payout: string;
                beneficiary: string;
            };
            Canceled: {
                index: string;
                childIndex: string;
            };
        };
    };
    /**
     * Lookup137: pallet_referenda::pallet::Event<T, I>
     **/
    PalletReferendaEvent: {
        _enum: {
            Submitted: {
                index: string;
                track: string;
                proposal: string;
            };
            DecisionDepositPlaced: {
                index: string;
                who: string;
                amount: string;
            };
            DecisionDepositRefunded: {
                index: string;
                who: string;
                amount: string;
            };
            DepositSlashed: {
                who: string;
                amount: string;
            };
            DecisionStarted: {
                index: string;
                track: string;
                proposal: string;
                tally: string;
            };
            ConfirmStarted: {
                index: string;
            };
            ConfirmAborted: {
                index: string;
            };
            Confirmed: {
                index: string;
                tally: string;
            };
            Approved: {
                index: string;
            };
            Rejected: {
                index: string;
                tally: string;
            };
            TimedOut: {
                index: string;
                tally: string;
            };
            Cancelled: {
                index: string;
                tally: string;
            };
            Killed: {
                index: string;
                tally: string;
            };
            SubmissionDepositRefunded: {
                index: string;
                who: string;
                amount: string;
            };
            MetadataSet: {
                _alias: {
                    hash_: string;
                };
                index: string;
                hash_: string;
            };
            MetadataCleared: {
                _alias: {
                    hash_: string;
                };
                index: string;
                hash_: string;
            };
        };
    };
    /**
     * Lookup138: frame_support::traits::preimages::Bounded<kitchensink_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>
     **/
    FrameSupportPreimagesBounded: {
        _enum: {
            Legacy: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            Inline: string;
            Lookup: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
                len: string;
            };
        };
    };
    /**
     * Lookup140: frame_system::pallet::Call<T>
     **/
    FrameSystemCall: {
        _enum: {
            remark: {
                remark: string;
            };
            set_heap_pages: {
                pages: string;
            };
            set_code: {
                code: string;
            };
            set_code_without_checks: {
                code: string;
            };
            set_storage: {
                items: string;
            };
            kill_storage: {
                _alias: {
                    keys_: string;
                };
                keys_: string;
            };
            kill_prefix: {
                prefix: string;
                subkeys: string;
            };
            remark_with_event: {
                remark: string;
            };
            __Unused8: string;
            authorize_upgrade: {
                codeHash: string;
            };
            authorize_upgrade_without_checks: {
                codeHash: string;
            };
            apply_authorized_upgrade: {
                code: string;
            };
        };
    };
    /**
     * Lookup144: pallet_utility::pallet::Call<T>
     **/
    PalletUtilityCall: {
        _enum: {
            batch: {
                calls: string;
            };
            as_derivative: {
                index: string;
                call: string;
            };
            batch_all: {
                calls: string;
            };
            dispatch_as: {
                asOrigin: string;
                call: string;
            };
            force_batch: {
                calls: string;
            };
            with_weight: {
                call: string;
                weight: string;
            };
            if_else: {
                main: string;
                fallback: string;
            };
            dispatch_as_fallible: {
                asOrigin: string;
                call: string;
            };
        };
    };
    /**
     * Lookup146: kitchensink_runtime::OriginCaller
     **/
    KitchensinkRuntimeOriginCaller: {
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
            Council: string;
            TechnicalCommittee: string;
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
            AllianceMotion: string;
        };
    };
    /**
     * Lookup147: frame_support::dispatch::RawOrigin<sp_core::crypto::AccountId32>
     **/
    FrameSupportDispatchRawOrigin: {
        _enum: {
            Root: string;
            Signed: string;
            None: string;
        };
    };
    /**
     * Lookup148: pallet_collective::RawOrigin<sp_core::crypto::AccountId32, I>
     **/
    PalletCollectiveRawOrigin: {
        _enum: {
            Members: string;
            Member: string;
            _Phantom: string;
        };
    };
    /**
     * Lookup151: pallet_babe::pallet::Call<T>
     **/
    PalletBabeCall: {
        _enum: {
            report_equivocation: {
                equivocationProof: string;
                keyOwnerProof: string;
            };
            report_equivocation_unsigned: {
                equivocationProof: string;
                keyOwnerProof: string;
            };
            plan_config_change: {
                config: string;
            };
        };
    };
    /**
     * Lookup152: sp_consensus_slots::EquivocationProof<sp_runtime::generic::header::Header<Number, Hash>, sp_consensus_babe::app::Public>
     **/
    SpConsensusSlotsEquivocationProof: {
        offender: string;
        slot: string;
        firstHeader: string;
        secondHeader: string;
    };
    /**
     * Lookup153: sp_runtime::generic::header::Header<Number, Hash>
     **/
    SpRuntimeHeader: {
        parentHash: string;
        number: string;
        stateRoot: string;
        extrinsicsRoot: string;
        digest: string;
    };
    /**
     * Lookup155: sp_consensus_babe::app::Public
     **/
    SpConsensusBabeAppPublic: string;
    /**
     * Lookup157: sp_session::MembershipProof
     **/
    SpSessionMembershipProof: {
        session: string;
        trieNodes: string;
        validatorCount: string;
    };
    /**
     * Lookup158: sp_consensus_babe::digests::NextConfigDescriptor
     **/
    SpConsensusBabeDigestsNextConfigDescriptor: {
        _enum: {
            __Unused0: string;
            V1: {
                c: string;
                allowedSlots: string;
            };
        };
    };
    /**
     * Lookup160: sp_consensus_babe::AllowedSlots
     **/
    SpConsensusBabeAllowedSlots: {
        _enum: string[];
    };
    /**
     * Lookup161: pallet_timestamp::pallet::Call<T>
     **/
    PalletTimestampCall: {
        _enum: {
            set: {
                now: string;
            };
        };
    };
    /**
     * Lookup162: pallet_indices::pallet::Call<T>
     **/
    PalletIndicesCall: {
        _enum: {
            claim: {
                index: string;
            };
            transfer: {
                _alias: {
                    new_: string;
                };
                new_: string;
                index: string;
            };
            free: {
                index: string;
            };
            force_transfer: {
                _alias: {
                    new_: string;
                };
                new_: string;
                index: string;
                freeze: string;
            };
            freeze: {
                index: string;
            };
            poke_deposit: {
                index: string;
            };
        };
    };
    /**
     * Lookup165: pallet_balances::pallet::Call<T, I>
     **/
    PalletBalancesCall: {
        _enum: {
            transfer_allow_death: {
                dest: string;
                value: string;
            };
            __Unused1: string;
            force_transfer: {
                source: string;
                dest: string;
                value: string;
            };
            transfer_keep_alive: {
                dest: string;
                value: string;
            };
            transfer_all: {
                dest: string;
                keepAlive: string;
            };
            force_unreserve: {
                who: string;
                amount: string;
            };
            upgrade_accounts: {
                who: string;
            };
            __Unused7: string;
            force_set_balance: {
                who: string;
                newFree: string;
            };
            force_adjust_total_issuance: {
                direction: string;
                delta: string;
            };
            burn: {
                value: string;
                keepAlive: string;
            };
        };
    };
    /**
     * Lookup167: pallet_balances::types::AdjustmentDirection
     **/
    PalletBalancesAdjustmentDirection: {
        _enum: string[];
    };
    /**
     * Lookup168: pallet_election_provider_multi_phase::pallet::Call<T>
     **/
    PalletElectionProviderMultiPhaseCall: {
        _enum: {
            submit_unsigned: {
                rawSolution: string;
                witness: string;
            };
            set_minimum_untrusted_score: {
                maybeNextScore: string;
            };
            set_emergency_election_result: {
                supports: string;
            };
            submit: {
                rawSolution: string;
            };
            governance_fallback: {
                maybeMaxVoters: string;
                maybeMaxTargets: string;
            };
        };
    };
    /**
     * Lookup169: pallet_election_provider_multi_phase::RawSolution<kitchensink_runtime::NposSolution16>
     **/
    PalletElectionProviderMultiPhaseRawSolution: {
        solution: string;
        score: string;
        round: string;
    };
    /**
     * Lookup170: kitchensink_runtime::NposSolution16
     **/
    KitchensinkRuntimeNposSolution16: {
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
     * Lookup221: pallet_election_provider_multi_phase::SolutionOrSnapshotSize
     **/
    PalletElectionProviderMultiPhaseSolutionOrSnapshotSize: {
        voters: string;
        targets: string;
    };
    /**
     * Lookup225: sp_npos_elections::Support<sp_core::crypto::AccountId32>
     **/
    SpNposElectionsSupport: {
        total: string;
        voters: string;
    };
    /**
     * Lookup226: pallet_staking::pallet::pallet::Call<T>
     **/
    PalletStakingPalletCall: {
        _enum: {
            bond: {
                value: string;
                payee: string;
            };
            bond_extra: {
                maxAdditional: string;
            };
            unbond: {
                value: string;
            };
            withdraw_unbonded: {
                numSlashingSpans: string;
            };
            validate: {
                prefs: string;
            };
            nominate: {
                targets: string;
            };
            chill: string;
            set_payee: {
                payee: string;
            };
            set_controller: string;
            set_validator_count: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            increase_validator_count: {
                additional: string;
            };
            scale_validator_count: {
                factor: string;
            };
            force_no_eras: string;
            force_new_era: string;
            set_invulnerables: {
                invulnerables: string;
            };
            force_unstake: {
                stash: string;
                numSlashingSpans: string;
            };
            force_new_era_always: string;
            cancel_deferred_slash: {
                era: string;
                slashIndices: string;
            };
            payout_stakers: {
                validatorStash: string;
                era: string;
            };
            rebond: {
                value: string;
            };
            reap_stash: {
                stash: string;
                numSlashingSpans: string;
            };
            kick: {
                who: string;
            };
            set_staking_configs: {
                minNominatorBond: string;
                minValidatorBond: string;
                maxNominatorCount: string;
                maxValidatorCount: string;
                chillThreshold: string;
                minCommission: string;
                maxStakedRewards: string;
            };
            chill_other: {
                stash: string;
            };
            force_apply_min_commission: {
                validatorStash: string;
            };
            set_min_commission: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            payout_stakers_by_page: {
                validatorStash: string;
                era: string;
                page: string;
            };
            update_payee: {
                controller: string;
            };
            deprecate_controller_batch: {
                controllers: string;
            };
            restore_ledger: {
                stash: string;
                maybeController: string;
                maybeTotal: string;
                maybeUnlocking: string;
            };
            migrate_currency: {
                stash: string;
            };
            __Unused31: string;
            __Unused32: string;
            manual_slash: {
                validatorStash: string;
                era: string;
                slashFraction: string;
            };
        };
    };
    /**
     * Lookup229: pallet_staking::pallet::pallet::ConfigOp<T>
     **/
    PalletStakingPalletConfigOpU128: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup230: pallet_staking::pallet::pallet::ConfigOp<T>
     **/
    PalletStakingPalletConfigOpU32: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup231: pallet_staking::pallet::pallet::ConfigOp<sp_arithmetic::per_things::Percent>
     **/
    PalletStakingPalletConfigOpPercent: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup232: pallet_staking::pallet::pallet::ConfigOp<sp_arithmetic::per_things::Perbill>
     **/
    PalletStakingPalletConfigOpPerbill: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup237: pallet_staking::UnlockChunk<Balance>
     **/
    PalletStakingUnlockChunk: {
        value: string;
        era: string;
    };
    /**
     * Lookup239: pallet_session::pallet::Call<T>
     **/
    PalletSessionCall: {
        _enum: {
            set_keys: {
                _alias: {
                    keys_: string;
                };
                keys_: string;
                proof: string;
            };
            purge_keys: string;
        };
    };
    /**
     * Lookup240: kitchensink_runtime::SessionKeys
     **/
    KitchensinkRuntimeSessionKeys: {
        grandpa: string;
        babe: string;
        imOnline: string;
        authorityDiscovery: string;
        mixnet: string;
        beefy: string;
    };
    /**
     * Lookup241: sp_authority_discovery::app::Public
     **/
    SpAuthorityDiscoveryAppPublic: string;
    /**
     * Lookup242: sp_mixnet::types::app::Public
     **/
    SpMixnetAppPublic: string;
    /**
     * Lookup243: sp_consensus_beefy::ecdsa_crypto::Public
     **/
    SpConsensusBeefyEcdsaCryptoPublic: string;
    /**
     * Lookup245: pallet_democracy::pallet::Call<T>
     **/
    PalletDemocracyCall: {
        _enum: {
            propose: {
                proposal: string;
                value: string;
            };
            second: {
                proposal: string;
            };
            vote: {
                refIndex: string;
                vote: string;
            };
            emergency_cancel: {
                refIndex: string;
            };
            external_propose: {
                proposal: string;
            };
            external_propose_majority: {
                proposal: string;
            };
            external_propose_default: {
                proposal: string;
            };
            fast_track: {
                proposalHash: string;
                votingPeriod: string;
                delay: string;
            };
            veto_external: {
                proposalHash: string;
            };
            cancel_referendum: {
                refIndex: string;
            };
            delegate: {
                to: string;
                conviction: string;
                balance: string;
            };
            undelegate: string;
            clear_public_proposals: string;
            unlock: {
                target: string;
            };
            remove_vote: {
                index: string;
            };
            remove_other_vote: {
                target: string;
                index: string;
            };
            blacklist: {
                proposalHash: string;
                maybeRefIndex: string;
            };
            cancel_proposal: {
                propIndex: string;
            };
            set_metadata: {
                owner: string;
                maybeHash: string;
            };
        };
    };
    /**
     * Lookup246: pallet_democracy::conviction::Conviction
     **/
    PalletDemocracyConviction: {
        _enum: string[];
    };
    /**
     * Lookup248: pallet_collective::pallet::Call<T, I>
     **/
    PalletCollectiveCall: {
        _enum: {
            set_members: {
                newMembers: string;
                prime: string;
                oldCount: string;
            };
            execute: {
                proposal: string;
                lengthBound: string;
            };
            propose: {
                threshold: string;
                proposal: string;
                lengthBound: string;
            };
            vote: {
                proposal: string;
                index: string;
                approve: string;
            };
            __Unused4: string;
            disapprove_proposal: {
                proposalHash: string;
            };
            close: {
                proposalHash: string;
                index: string;
                proposalWeightBound: string;
                lengthBound: string;
            };
            kill: {
                proposalHash: string;
            };
            release_proposal_cost: {
                proposalHash: string;
            };
        };
    };
    /**
     * Lookup250: pallet_elections_phragmen::pallet::Call<T>
     **/
    PalletElectionsPhragmenCall: {
        _enum: {
            vote: {
                votes: string;
                value: string;
            };
            remove_voter: string;
            submit_candidacy: {
                candidateCount: string;
            };
            renounce_candidacy: {
                renouncing: string;
            };
            remove_member: {
                who: string;
                slashBond: string;
                rerunElection: string;
            };
            clean_defunct_voters: {
                numVoters: string;
                numDefunct: string;
            };
        };
    };
    /**
     * Lookup251: pallet_elections_phragmen::Renouncing
     **/
    PalletElectionsPhragmenRenouncing: {
        _enum: {
            Member: string;
            RunnerUp: string;
            Candidate: string;
        };
    };
    /**
     * Lookup252: pallet_membership::pallet::Call<T, I>
     **/
    PalletMembershipCall: {
        _enum: {
            add_member: {
                who: string;
            };
            remove_member: {
                who: string;
            };
            swap_member: {
                remove: string;
                add: string;
            };
            reset_members: {
                members: string;
            };
            change_key: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            set_prime: {
                who: string;
            };
            clear_prime: string;
        };
    };
    /**
     * Lookup253: pallet_grandpa::pallet::Call<T>
     **/
    PalletGrandpaCall: {
        _enum: {
            report_equivocation: {
                equivocationProof: string;
                keyOwnerProof: string;
            };
            report_equivocation_unsigned: {
                equivocationProof: string;
                keyOwnerProof: string;
            };
            note_stalled: {
                delay: string;
                bestFinalizedBlockNumber: string;
            };
        };
    };
    /**
     * Lookup254: sp_consensus_grandpa::EquivocationProof<primitive_types::H256, N>
     **/
    SpConsensusGrandpaEquivocationProof: {
        setId: string;
        equivocation: string;
    };
    /**
     * Lookup255: sp_consensus_grandpa::Equivocation<primitive_types::H256, N>
     **/
    SpConsensusGrandpaEquivocation: {
        _enum: {
            Prevote: string;
            Precommit: string;
        };
    };
    /**
     * Lookup256: finality_grandpa::Equivocation<sp_consensus_grandpa::app::Public, finality_grandpa::Prevote<primitive_types::H256, N>, sp_consensus_grandpa::app::Signature>
     **/
    FinalityGrandpaEquivocationPrevote: {
        roundNumber: string;
        identity: string;
        first: string;
        second: string;
    };
    /**
     * Lookup257: finality_grandpa::Prevote<primitive_types::H256, N>
     **/
    FinalityGrandpaPrevote: {
        targetHash: string;
        targetNumber: string;
    };
    /**
     * Lookup258: sp_consensus_grandpa::app::Signature
     **/
    SpConsensusGrandpaAppSignature: string;
    /**
     * Lookup261: finality_grandpa::Equivocation<sp_consensus_grandpa::app::Public, finality_grandpa::Precommit<primitive_types::H256, N>, sp_consensus_grandpa::app::Signature>
     **/
    FinalityGrandpaEquivocationPrecommit: {
        roundNumber: string;
        identity: string;
        first: string;
        second: string;
    };
    /**
     * Lookup262: finality_grandpa::Precommit<primitive_types::H256, N>
     **/
    FinalityGrandpaPrecommit: {
        targetHash: string;
        targetNumber: string;
    };
    /**
     * Lookup264: pallet_treasury::pallet::Call<T, I>
     **/
    PalletTreasuryCall: {
        _enum: {
            __Unused0: string;
            __Unused1: string;
            __Unused2: string;
            spend_local: {
                amount: string;
                beneficiary: string;
            };
            remove_approval: {
                proposalId: string;
            };
            spend: {
                assetKind: string;
                amount: string;
                beneficiary: string;
                validFrom: string;
            };
            payout: {
                index: string;
            };
            check_status: {
                index: string;
            };
            void_spend: {
                index: string;
            };
        };
    };
    /**
     * Lookup265: pallet_asset_rate::pallet::Call<T>
     **/
    PalletAssetRateCall: {
        _enum: {
            create: {
                assetKind: string;
                rate: string;
            };
            update: {
                assetKind: string;
                rate: string;
            };
            remove: {
                assetKind: string;
            };
        };
    };
    /**
     * Lookup266: pallet_contracts::pallet::Call<T>
     **/
    PalletContractsCall: {
        _enum: {
            call_old_weight: {
                dest: string;
                value: string;
                gasLimit: string;
                storageDepositLimit: string;
                data: string;
            };
            instantiate_with_code_old_weight: {
                value: string;
                gasLimit: string;
                storageDepositLimit: string;
                code: string;
                data: string;
                salt: string;
            };
            instantiate_old_weight: {
                value: string;
                gasLimit: string;
                storageDepositLimit: string;
                codeHash: string;
                data: string;
                salt: string;
            };
            upload_code: {
                code: string;
                storageDepositLimit: string;
                determinism: string;
            };
            remove_code: {
                codeHash: string;
            };
            set_code: {
                dest: string;
                codeHash: string;
            };
            call: {
                dest: string;
                value: string;
                gasLimit: string;
                storageDepositLimit: string;
                data: string;
            };
            instantiate_with_code: {
                value: string;
                gasLimit: string;
                storageDepositLimit: string;
                code: string;
                data: string;
                salt: string;
            };
            instantiate: {
                value: string;
                gasLimit: string;
                storageDepositLimit: string;
                codeHash: string;
                data: string;
                salt: string;
            };
            migrate: {
                weightLimit: string;
            };
        };
    };
    /**
     * Lookup268: pallet_contracts::wasm::Determinism
     **/
    PalletContractsWasmDeterminism: {
        _enum: string[];
    };
    /**
     * Lookup269: pallet_sudo::pallet::Call<T>
     **/
    PalletSudoCall: {
        _enum: {
            sudo: {
                call: string;
            };
            sudo_unchecked_weight: {
                call: string;
                weight: string;
            };
            set_key: {
                _alias: {
                    new_: string;
                };
                new_: string;
            };
            sudo_as: {
                who: string;
                call: string;
            };
            remove_key: string;
        };
    };
    /**
     * Lookup270: pallet_im_online::pallet::Call<T>
     **/
    PalletImOnlineCall: {
        _enum: {
            heartbeat: {
                heartbeat: string;
                signature: string;
            };
        };
    };
    /**
     * Lookup271: pallet_im_online::Heartbeat<BlockNumber>
     **/
    PalletImOnlineHeartbeat: {
        blockNumber: string;
        sessionIndex: string;
        authorityIndex: string;
        validatorsLen: string;
    };
    /**
     * Lookup272: pallet_im_online::sr25519::app_sr25519::Signature
     **/
    PalletImOnlineSr25519AppSr25519Signature: string;
    /**
     * Lookup273: pallet_identity::pallet::Call<T>
     **/
    PalletIdentityCall: {
        _enum: {
            add_registrar: {
                account: string;
            };
            set_identity: {
                info: string;
            };
            set_subs: {
                subs: string;
            };
            clear_identity: string;
            request_judgement: {
                regIndex: string;
                maxFee: string;
            };
            cancel_request: {
                regIndex: string;
            };
            set_fee: {
                index: string;
                fee: string;
            };
            set_account_id: {
                _alias: {
                    new_: string;
                };
                index: string;
                new_: string;
            };
            set_fields: {
                index: string;
                fields: string;
            };
            provide_judgement: {
                regIndex: string;
                target: string;
                judgement: string;
                identity: string;
            };
            kill_identity: {
                target: string;
            };
            add_sub: {
                sub: string;
                data: string;
            };
            rename_sub: {
                sub: string;
                data: string;
            };
            remove_sub: {
                sub: string;
            };
            quit_sub: string;
            add_username_authority: {
                authority: string;
                suffix: string;
                allocation: string;
            };
            remove_username_authority: {
                suffix: string;
                authority: string;
            };
            set_username_for: {
                who: string;
                username: string;
                signature: string;
                useAllocation: string;
            };
            accept_username: {
                username: string;
            };
            remove_expired_approval: {
                username: string;
            };
            set_primary_username: {
                username: string;
            };
            unbind_username: {
                username: string;
            };
            remove_username: {
                username: string;
            };
            kill_username: {
                username: string;
            };
        };
    };
    /**
     * Lookup274: pallet_identity::legacy::IdentityInfo<FieldLimit>
     **/
    PalletIdentityLegacyIdentityInfo: {
        additional: string;
        display: string;
        legal: string;
        web: string;
        riot: string;
        email: string;
        pgpFingerprint: string;
        image: string;
        twitter: string;
    };
    /**
     * Lookup311: pallet_identity::types::Judgement<Balance>
     **/
    PalletIdentityJudgement: {
        _enum: {
            Unknown: string;
            FeePaid: string;
            Reasonable: string;
            KnownGood: string;
            OutOfDate: string;
            LowQuality: string;
            Erroneous: string;
        };
    };
    /**
     * Lookup313: sp_runtime::MultiSignature
     **/
    SpRuntimeMultiSignature: {
        _enum: {
            Ed25519: string;
            Sr25519: string;
            Ecdsa: string;
        };
    };
    /**
     * Lookup315: pallet_society::pallet::Call<T, I>
     **/
    PalletSocietyCall: {
        _enum: {
            bid: {
                value: string;
            };
            unbid: string;
            vouch: {
                who: string;
                value: string;
                tip: string;
            };
            unvouch: string;
            vote: {
                candidate: string;
                approve: string;
            };
            defender_vote: {
                approve: string;
            };
            payout: string;
            waive_repay: {
                amount: string;
            };
            found_society: {
                founder: string;
                maxMembers: string;
                maxIntake: string;
                maxStrikes: string;
                candidateDeposit: string;
                rules: string;
            };
            dissolve: string;
            judge_suspended_member: {
                who: string;
                forgive: string;
            };
            set_parameters: {
                maxMembers: string;
                maxIntake: string;
                maxStrikes: string;
                candidateDeposit: string;
            };
            punish_skeptic: string;
            claim_membership: string;
            bestow_membership: {
                candidate: string;
            };
            kick_candidate: {
                candidate: string;
            };
            resign_candidacy: string;
            drop_candidate: {
                candidate: string;
            };
            cleanup_candidacy: {
                candidate: string;
                max: string;
            };
            cleanup_challenge: {
                challengeRound: string;
                max: string;
            };
        };
    };
    /**
     * Lookup316: pallet_recovery::pallet::Call<T>
     **/
    PalletRecoveryCall: {
        _enum: {
            as_recovered: {
                account: string;
                call: string;
            };
            set_recovered: {
                lost: string;
                rescuer: string;
            };
            create_recovery: {
                friends: string;
                threshold: string;
                delayPeriod: string;
            };
            initiate_recovery: {
                account: string;
            };
            vouch_recovery: {
                lost: string;
                rescuer: string;
            };
            claim_recovery: {
                account: string;
            };
            close_recovery: {
                rescuer: string;
            };
            remove_recovery: string;
            cancel_recovered: {
                account: string;
            };
        };
    };
    /**
     * Lookup317: pallet_vesting::pallet::Call<T>
     **/
    PalletVestingCall: {
        _enum: {
            vest: string;
            vest_other: {
                target: string;
            };
            vested_transfer: {
                target: string;
                schedule: string;
            };
            force_vested_transfer: {
                source: string;
                target: string;
                schedule: string;
            };
            merge_schedules: {
                schedule1Index: string;
                schedule2Index: string;
            };
            force_remove_vesting_schedule: {
                target: string;
                scheduleIndex: string;
            };
        };
    };
    /**
     * Lookup318: pallet_vesting::vesting_info::VestingInfo<Balance, BlockNumber>
     **/
    PalletVestingVestingInfo: {
        locked: string;
        perBlock: string;
        startingBlock: string;
    };
    /**
     * Lookup319: pallet_scheduler::pallet::Call<T>
     **/
    PalletSchedulerCall: {
        _enum: {
            schedule: {
                when: string;
                maybePeriodic: string;
                priority: string;
                call: string;
            };
            cancel: {
                when: string;
                index: string;
            };
            schedule_named: {
                id: string;
                when: string;
                maybePeriodic: string;
                priority: string;
                call: string;
            };
            cancel_named: {
                id: string;
            };
            schedule_after: {
                after: string;
                maybePeriodic: string;
                priority: string;
                call: string;
            };
            schedule_named_after: {
                id: string;
                after: string;
                maybePeriodic: string;
                priority: string;
                call: string;
            };
            set_retry: {
                task: string;
                retries: string;
                period: string;
            };
            set_retry_named: {
                id: string;
                retries: string;
                period: string;
            };
            cancel_retry: {
                task: string;
            };
            cancel_retry_named: {
                id: string;
            };
        };
    };
    /**
     * Lookup321: pallet_glutton::pallet::Call<T>
     **/
    PalletGluttonCall: {
        _enum: {
            initialize_pallet: {
                newCount: string;
                witnessCount: string;
            };
            set_compute: {
                compute: string;
            };
            set_storage: {
                storage: string;
            };
            bloat: {
                garbage: string;
            };
            set_block_length: {
                blockLength: string;
            };
        };
    };
    /**
     * Lookup324: pallet_preimage::pallet::Call<T>
     **/
    PalletPreimageCall: {
        _enum: {
            note_preimage: {
                bytes: string;
            };
            unnote_preimage: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            request_preimage: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            unrequest_preimage: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            ensure_updated: {
                hashes: string;
            };
        };
    };
    /**
     * Lookup326: pallet_proxy::pallet::Call<T>
     **/
    PalletProxyCall: {
        _enum: {
            proxy: {
                real: string;
                forceProxyType: string;
                call: string;
            };
            add_proxy: {
                delegate: string;
                proxyType: string;
                delay: string;
            };
            remove_proxy: {
                delegate: string;
                proxyType: string;
                delay: string;
            };
            remove_proxies: string;
            create_pure: {
                proxyType: string;
                delay: string;
                index: string;
            };
            kill_pure: {
                spawner: string;
                proxyType: string;
                index: string;
                height: string;
                extIndex: string;
            };
            announce: {
                real: string;
                callHash: string;
            };
            remove_announcement: {
                real: string;
                callHash: string;
            };
            reject_announcement: {
                delegate: string;
                callHash: string;
            };
            proxy_announced: {
                delegate: string;
                real: string;
                forceProxyType: string;
                call: string;
            };
            poke_deposit: string;
        };
    };
    /**
     * Lookup328: pallet_multisig::pallet::Call<T>
     **/
    PalletMultisigCall: {
        _enum: {
            as_multi_threshold_1: {
                otherSignatories: string;
                call: string;
            };
            as_multi: {
                threshold: string;
                otherSignatories: string;
                maybeTimepoint: string;
                call: string;
                maxWeight: string;
            };
            approve_as_multi: {
                threshold: string;
                otherSignatories: string;
                maybeTimepoint: string;
                callHash: string;
                maxWeight: string;
            };
            cancel_as_multi: {
                threshold: string;
                otherSignatories: string;
                timepoint: string;
                callHash: string;
            };
            poke_deposit: {
                threshold: string;
                otherSignatories: string;
                callHash: string;
            };
        };
    };
    /**
     * Lookup330: pallet_bounties::pallet::Call<T, I>
     **/
    PalletBountiesCall: {
        _enum: {
            propose_bounty: {
                value: string;
                description: string;
            };
            approve_bounty: {
                bountyId: string;
            };
            propose_curator: {
                bountyId: string;
                curator: string;
                fee: string;
            };
            unassign_curator: {
                bountyId: string;
            };
            accept_curator: {
                bountyId: string;
            };
            award_bounty: {
                bountyId: string;
                beneficiary: string;
            };
            claim_bounty: {
                bountyId: string;
            };
            close_bounty: {
                bountyId: string;
            };
            extend_bounty_expiry: {
                bountyId: string;
                remark: string;
            };
            approve_bounty_with_curator: {
                bountyId: string;
                curator: string;
                fee: string;
            };
        };
    };
    /**
     * Lookup331: pallet_tips::pallet::Call<T, I>
     **/
    PalletTipsCall: {
        _enum: {
            report_awesome: {
                reason: string;
                who: string;
            };
            retract_tip: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            tip_new: {
                reason: string;
                who: string;
                tipValue: string;
            };
            tip: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
                tipValue: string;
            };
            close_tip: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
            slash_tip: {
                _alias: {
                    hash_: string;
                };
                hash_: string;
            };
        };
    };
    /**
     * Lookup332: pallet_assets::pallet::Call<T, I>
     **/
    PalletAssetsCall: {
        _enum: {
            create: {
                id: string;
                admin: string;
                minBalance: string;
            };
            force_create: {
                id: string;
                owner: string;
                isSufficient: string;
                minBalance: string;
            };
            start_destroy: {
                id: string;
            };
            destroy_accounts: {
                id: string;
            };
            destroy_approvals: {
                id: string;
            };
            finish_destroy: {
                id: string;
            };
            mint: {
                id: string;
                beneficiary: string;
                amount: string;
            };
            burn: {
                id: string;
                who: string;
                amount: string;
            };
            transfer: {
                id: string;
                target: string;
                amount: string;
            };
            transfer_keep_alive: {
                id: string;
                target: string;
                amount: string;
            };
            force_transfer: {
                id: string;
                source: string;
                dest: string;
                amount: string;
            };
            freeze: {
                id: string;
                who: string;
            };
            thaw: {
                id: string;
                who: string;
            };
            freeze_asset: {
                id: string;
            };
            thaw_asset: {
                id: string;
            };
            transfer_ownership: {
                id: string;
                owner: string;
            };
            set_team: {
                id: string;
                issuer: string;
                admin: string;
                freezer: string;
            };
            set_metadata: {
                id: string;
                name: string;
                symbol: string;
                decimals: string;
            };
            clear_metadata: {
                id: string;
            };
            force_set_metadata: {
                id: string;
                name: string;
                symbol: string;
                decimals: string;
                isFrozen: string;
            };
            force_clear_metadata: {
                id: string;
            };
            force_asset_status: {
                id: string;
                owner: string;
                issuer: string;
                admin: string;
                freezer: string;
                minBalance: string;
                isSufficient: string;
                isFrozen: string;
            };
            approve_transfer: {
                id: string;
                delegate: string;
                amount: string;
            };
            cancel_approval: {
                id: string;
                delegate: string;
            };
            force_cancel_approval: {
                id: string;
                owner: string;
                delegate: string;
            };
            transfer_approved: {
                id: string;
                owner: string;
                destination: string;
                amount: string;
            };
            touch: {
                id: string;
            };
            refund: {
                id: string;
                allowBurn: string;
            };
            set_min_balance: {
                id: string;
                minBalance: string;
            };
            touch_other: {
                id: string;
                who: string;
            };
            refund_other: {
                id: string;
                who: string;
            };
            block: {
                id: string;
                who: string;
            };
            transfer_all: {
                id: string;
                dest: string;
                keepAlive: string;
            };
        };
    };
    /**
     * Lookup334: pallet_beefy::pallet::Call<T>
     **/
    PalletBeefyCall: {
        _enum: {
            report_double_voting: {
                equivocationProof: string;
                keyOwnerProof: string;
            };
            report_double_voting_unsigned: {
                equivocationProof: string;
                keyOwnerProof: string;
            };
            set_new_genesis: {
                delayInBlocks: string;
            };
            report_fork_voting: {
                equivocationProof: string;
                keyOwnerProof: string;
            };
            report_fork_voting_unsigned: {
                equivocationProof: string;
                keyOwnerProof: string;
            };
            report_future_block_voting: {
                equivocationProof: string;
                keyOwnerProof: string;
            };
            report_future_block_voting_unsigned: {
                equivocationProof: string;
                keyOwnerProof: string;
            };
        };
    };
    /**
     * Lookup335: sp_consensus_beefy::DoubleVotingProof<Number, sp_consensus_beefy::ecdsa_crypto::Public, sp_consensus_beefy::ecdsa_crypto::Signature>
     **/
    SpConsensusBeefyDoubleVotingProof: {
        first: string;
        second: string;
    };
    /**
     * Lookup336: sp_consensus_beefy::ecdsa_crypto::Signature
     **/
    SpConsensusBeefyEcdsaCryptoSignature: string;
    /**
     * Lookup337: sp_consensus_beefy::VoteMessage<Number, sp_consensus_beefy::ecdsa_crypto::Public, sp_consensus_beefy::ecdsa_crypto::Signature>
     **/
    SpConsensusBeefyVoteMessage: {
        commitment: string;
        id: string;
        signature: string;
    };
    /**
     * Lookup338: sp_consensus_beefy::commitment::Commitment<TBlockNumber>
     **/
    SpConsensusBeefyCommitment: {
        payload: string;
        blockNumber: string;
        validatorSetId: string;
    };
    /**
     * Lookup339: sp_consensus_beefy::payload::Payload
     **/
    SpConsensusBeefyPayload: string;
    /**
     * Lookup342: sp_consensus_beefy::ForkVotingProof<sp_runtime::generic::header::Header<Number, Hash>, sp_consensus_beefy::ecdsa_crypto::Public, sp_mmr_primitives::AncestryProof<primitive_types::H256>>
     **/
    SpConsensusBeefyForkVotingProofAncestryProof: {
        vote: string;
        ancestryProof: string;
        header: string;
    };
    /**
     * Lookup343: sp_mmr_primitives::AncestryProof<primitive_types::H256>
     **/
    SpMmrPrimitivesAncestryProof: {
        prevPeaks: string;
        prevLeafCount: string;
        leafCount: string;
        items: string;
    };
    /**
     * Lookup346: sp_consensus_beefy::FutureBlockVotingProof<Number, sp_consensus_beefy::ecdsa_crypto::Public>
     **/
    SpConsensusBeefyFutureBlockVotingProof: {
        vote: string;
    };
    /**
     * Lookup347: pallet_lottery::pallet::Call<T>
     **/
    PalletLotteryCall: {
        _enum: {
            buy_ticket: {
                call: string;
            };
            set_calls: {
                calls: string;
            };
            start_lottery: {
                price: string;
                length: string;
                delay: string;
                repeat: string;
            };
            stop_repeat: string;
        };
    };
    /**
     * Lookup348: pallet_nis::pallet::Call<T>
     **/
    PalletNisCall: {
        _enum: {
            place_bid: {
                amount: string;
                duration: string;
            };
            retract_bid: {
                amount: string;
                duration: string;
            };
            fund_deficit: string;
            thaw_private: {
                index: string;
                maybeProportion: string;
            };
            thaw_communal: {
                index: string;
            };
            communify: {
                index: string;
            };
            privatize: {
                index: string;
            };
        };
    };
    /**
     * Lookup350: pallet_uniques::pallet::Call<T, I>
     **/
    PalletUniquesCall: {
        _enum: {
            create: {
                collection: string;
                admin: string;
            };
            force_create: {
                collection: string;
                owner: string;
                freeHolding: string;
            };
            destroy: {
                collection: string;
                witness: string;
            };
            mint: {
                collection: string;
                item: string;
                owner: string;
            };
            burn: {
                collection: string;
                item: string;
                checkOwner: string;
            };
            transfer: {
                collection: string;
                item: string;
                dest: string;
            };
            redeposit: {
                collection: string;
                items: string;
            };
            freeze: {
                collection: string;
                item: string;
            };
            thaw: {
                collection: string;
                item: string;
            };
            freeze_collection: {
                collection: string;
            };
            thaw_collection: {
                collection: string;
            };
            transfer_ownership: {
                collection: string;
                newOwner: string;
            };
            set_team: {
                collection: string;
                issuer: string;
                admin: string;
                freezer: string;
            };
            approve_transfer: {
                collection: string;
                item: string;
                delegate: string;
            };
            cancel_approval: {
                collection: string;
                item: string;
                maybeCheckDelegate: string;
            };
            force_item_status: {
                collection: string;
                owner: string;
                issuer: string;
                admin: string;
                freezer: string;
                freeHolding: string;
                isFrozen: string;
            };
            set_attribute: {
                collection: string;
                maybeItem: string;
                key: string;
                value: string;
            };
            clear_attribute: {
                collection: string;
                maybeItem: string;
                key: string;
            };
            set_metadata: {
                collection: string;
                item: string;
                data: string;
                isFrozen: string;
            };
            clear_metadata: {
                collection: string;
                item: string;
            };
            set_collection_metadata: {
                collection: string;
                data: string;
                isFrozen: string;
            };
            clear_collection_metadata: {
                collection: string;
            };
            set_accept_ownership: {
                maybeCollection: string;
            };
            set_collection_max_supply: {
                collection: string;
                maxSupply: string;
            };
            set_price: {
                collection: string;
                item: string;
                price: string;
                whitelistedBuyer: string;
            };
            buy_item: {
                collection: string;
                item: string;
                bidPrice: string;
            };
        };
    };
    /**
     * Lookup351: pallet_uniques::types::DestroyWitness
     **/
    PalletUniquesDestroyWitness: {
        items: string;
        itemMetadatas: string;
        attributes: string;
    };
    /**
     * Lookup353: pallet_nfts::pallet::Call<T, I>
     **/
    PalletNftsCall: {
        _enum: {
            create: {
                admin: string;
                config: string;
            };
            force_create: {
                owner: string;
                config: string;
            };
            destroy: {
                collection: string;
                witness: string;
            };
            mint: {
                collection: string;
                item: string;
                mintTo: string;
                witnessData: string;
            };
            force_mint: {
                collection: string;
                item: string;
                mintTo: string;
                itemConfig: string;
            };
            burn: {
                collection: string;
                item: string;
            };
            transfer: {
                collection: string;
                item: string;
                dest: string;
            };
            redeposit: {
                collection: string;
                items: string;
            };
            lock_item_transfer: {
                collection: string;
                item: string;
            };
            unlock_item_transfer: {
                collection: string;
                item: string;
            };
            lock_collection: {
                collection: string;
                lockSettings: string;
            };
            transfer_ownership: {
                collection: string;
                newOwner: string;
            };
            set_team: {
                collection: string;
                issuer: string;
                admin: string;
                freezer: string;
            };
            force_collection_owner: {
                collection: string;
                owner: string;
            };
            force_collection_config: {
                collection: string;
                config: string;
            };
            approve_transfer: {
                collection: string;
                item: string;
                delegate: string;
                maybeDeadline: string;
            };
            cancel_approval: {
                collection: string;
                item: string;
                delegate: string;
            };
            clear_all_transfer_approvals: {
                collection: string;
                item: string;
            };
            lock_item_properties: {
                collection: string;
                item: string;
                lockMetadata: string;
                lockAttributes: string;
            };
            set_attribute: {
                collection: string;
                maybeItem: string;
                namespace: string;
                key: string;
                value: string;
            };
            force_set_attribute: {
                setAs: string;
                collection: string;
                maybeItem: string;
                namespace: string;
                key: string;
                value: string;
            };
            clear_attribute: {
                collection: string;
                maybeItem: string;
                namespace: string;
                key: string;
            };
            approve_item_attributes: {
                collection: string;
                item: string;
                delegate: string;
            };
            cancel_item_attributes_approval: {
                collection: string;
                item: string;
                delegate: string;
                witness: string;
            };
            set_metadata: {
                collection: string;
                item: string;
                data: string;
            };
            clear_metadata: {
                collection: string;
                item: string;
            };
            set_collection_metadata: {
                collection: string;
                data: string;
            };
            clear_collection_metadata: {
                collection: string;
            };
            set_accept_ownership: {
                maybeCollection: string;
            };
            set_collection_max_supply: {
                collection: string;
                maxSupply: string;
            };
            update_mint_settings: {
                collection: string;
                mintSettings: string;
            };
            set_price: {
                collection: string;
                item: string;
                price: string;
                whitelistedBuyer: string;
            };
            buy_item: {
                collection: string;
                item: string;
                bidPrice: string;
            };
            pay_tips: {
                tips: string;
            };
            create_swap: {
                offeredCollection: string;
                offeredItem: string;
                desiredCollection: string;
                maybeDesiredItem: string;
                maybePrice: string;
                duration: string;
            };
            cancel_swap: {
                offeredCollection: string;
                offeredItem: string;
            };
            claim_swap: {
                sendCollection: string;
                sendItem: string;
                receiveCollection: string;
                receiveItem: string;
                witnessPrice: string;
            };
            mint_pre_signed: {
                mintData: string;
                signature: string;
                signer: string;
            };
            set_attributes_pre_signed: {
                data: string;
                signature: string;
                signer: string;
            };
        };
    };
    /**
     * Lookup354: pallet_nfts::types::CollectionConfig<Price, BlockNumber, CollectionId>
     **/
    PalletNftsCollectionConfig: {
        settings: string;
        maxSupply: string;
        mintSettings: string;
    };
    /**
     * Lookup356: pallet_nfts::types::CollectionSetting
     **/
    PalletNftsCollectionSetting: {
        _enum: string[];
    };
    /**
     * Lookup357: pallet_nfts::types::MintSettings<Price, BlockNumber, CollectionId>
     **/
    PalletNftsMintSettings: {
        mintType: string;
        price: string;
        startBlock: string;
        endBlock: string;
        defaultItemSettings: string;
    };
    /**
     * Lookup358: pallet_nfts::types::MintType<CollectionId>
     **/
    PalletNftsMintType: {
        _enum: {
            Issuer: string;
            Public: string;
            HolderOf: string;
        };
    };
    /**
     * Lookup360: pallet_nfts::types::ItemSetting
     **/
    PalletNftsItemSetting: {
        _enum: string[];
    };
    /**
     * Lookup361: pallet_nfts::types::DestroyWitness
     **/
    PalletNftsDestroyWitness: {
        itemMetadatas: string;
        itemConfigs: string;
        attributes: string;
    };
    /**
     * Lookup363: pallet_nfts::types::MintWitness<ItemId, Balance>
     **/
    PalletNftsMintWitness: {
        ownedItem: string;
        mintPrice: string;
    };
    /**
     * Lookup364: pallet_nfts::types::ItemConfig
     **/
    PalletNftsItemConfig: {
        settings: string;
    };
    /**
     * Lookup365: pallet_nfts::types::CancelAttributesApprovalWitness
     **/
    PalletNftsCancelAttributesApprovalWitness: {
        accountAttributes: string;
    };
    /**
     * Lookup367: pallet_nfts::types::ItemTip<CollectionId, ItemId, sp_core::crypto::AccountId32, Amount>
     **/
    PalletNftsItemTip: {
        collection: string;
        item: string;
        receiver: string;
        amount: string;
    };
    /**
     * Lookup369: pallet_nfts::types::PreSignedMint<CollectionId, ItemId, sp_core::crypto::AccountId32, Deadline, Balance>
     **/
    PalletNftsPreSignedMint: {
        collection: string;
        item: string;
        attributes: string;
        metadata: string;
        onlyAccount: string;
        deadline: string;
        mintPrice: string;
    };
    /**
     * Lookup370: pallet_nfts::types::PreSignedAttributes<CollectionId, ItemId, sp_core::crypto::AccountId32, Deadline>
     **/
    PalletNftsPreSignedAttributes: {
        collection: string;
        item: string;
        attributes: string;
        namespace: string;
        deadline: string;
    };
    /**
     * Lookup371: pallet_nft_fractionalization::pallet::Call<T>
     **/
    PalletNftFractionalizationCall: {
        _enum: {
            fractionalize: {
                nftCollectionId: string;
                nftId: string;
                assetId: string;
                beneficiary: string;
                fractions: string;
            };
            unify: {
                nftCollectionId: string;
                nftId: string;
                assetId: string;
                beneficiary: string;
            };
        };
    };
    /**
     * Lookup372: pallet_salary::pallet::Call<T, I>
     **/
    PalletSalaryCall: {
        _enum: {
            init: string;
            bump: string;
            induct: string;
            register: string;
            payout: string;
            payout_other: {
                beneficiary: string;
            };
            check_payment: string;
        };
    };
    /**
     * Lookup373: pallet_core_fellowship::pallet::Call<T, I>
     **/
    PalletCoreFellowshipCall: {
        _enum: {
            bump: {
                who: string;
            };
            set_params: {
                params: string;
            };
            set_active: {
                isActive: string;
            };
            approve: {
                who: string;
                atRank: string;
            };
            induct: {
                who: string;
            };
            promote: {
                who: string;
                toRank: string;
            };
            offboard: {
                who: string;
            };
            submit_evidence: {
                wish: string;
                evidence: string;
            };
            import: string;
            set_partial_params: {
                partialParams: string;
            };
            promote_fast: {
                who: string;
                toRank: string;
            };
            import_member: {
                who: string;
            };
        };
    };
    /**
     * Lookup374: pallet_core_fellowship::ParamsType<Option<T>, Option<T>, Ranks>
     **/
    PalletCoreFellowshipParamsTypeOption: {
        activeSalary: string;
        passiveSalary: string;
        demotionPeriod: string;
        minPromotionPeriod: string;
        offboardTimeout: string;
    };
    /**
     * Lookup379: pallet_transaction_storage::pallet::Call<T>
     **/
    PalletTransactionStorageCall: {
        _enum: {
            store: {
                data: string;
            };
            renew: {
                block: string;
                index: string;
            };
            check_proof: {
                proof: string;
            };
        };
    };
    /**
     * Lookup380: sp_transaction_storage_proof::TransactionStorageProof
     **/
    SpTransactionStorageProofTransactionStorageProof: {
        chunk: string;
        proof: string;
    };
    /**
     * Lookup381: pallet_bags_list::pallet::Call<T, I>
     **/
    PalletBagsListCall: {
        _enum: {
            rebag: {
                dislocated: string;
            };
            put_in_front_of: {
                lighter: string;
            };
            put_in_front_of_other: {
                heavier: string;
                lighter: string;
            };
        };
    };
    /**
     * Lookup382: pallet_state_trie_migration::pallet::Call<T>
     **/
    PalletStateTrieMigrationCall: {
        _enum: {
            control_auto_migration: {
                maybeConfig: string;
            };
            continue_migrate: {
                limits: string;
                realSizeUpper: string;
                witnessTask: string;
            };
            migrate_custom_top: {
                _alias: {
                    keys_: string;
                };
                keys_: string;
                witnessSize: string;
            };
            migrate_custom_child: {
                root: string;
                childKeys: string;
                totalSize: string;
            };
            set_signed_max_limits: {
                limits: string;
            };
            force_set_progress: {
                progressTop: string;
                progressChild: string;
            };
        };
    };
    /**
     * Lookup384: pallet_state_trie_migration::pallet::MigrationLimits
     **/
    PalletStateTrieMigrationMigrationLimits: {
        _alias: {
            size_: string;
        };
        size_: string;
        item: string;
    };
    /**
     * Lookup385: pallet_state_trie_migration::pallet::MigrationTask<T>
     **/
    PalletStateTrieMigrationMigrationTask: {
        _alias: {
            size_: string;
        };
        progressTop: string;
        progressChild: string;
        size_: string;
        topItems: string;
        childItems: string;
    };
    /**
     * Lookup386: pallet_state_trie_migration::pallet::Progress<MaxKeyLen>
     **/
    PalletStateTrieMigrationProgress: {
        _enum: {
            ToStart: string;
            LastKey: string;
            Complete: string;
        };
    };
    /**
     * Lookup388: pallet_child_bounties::pallet::Call<T>
     **/
    PalletChildBountiesCall: {
        _enum: {
            add_child_bounty: {
                parentBountyId: string;
                value: string;
                description: string;
            };
            propose_curator: {
                parentBountyId: string;
                childBountyId: string;
                curator: string;
                fee: string;
            };
            accept_curator: {
                parentBountyId: string;
                childBountyId: string;
            };
            unassign_curator: {
                parentBountyId: string;
                childBountyId: string;
            };
            award_child_bounty: {
                parentBountyId: string;
                childBountyId: string;
                beneficiary: string;
            };
            claim_child_bounty: {
                parentBountyId: string;
                childBountyId: string;
            };
            close_child_bounty: {
                parentBountyId: string;
                childBountyId: string;
            };
        };
    };
    /**
     * Lookup389: pallet_referenda::pallet::Call<T, I>
     **/
    PalletReferendaCall: {
        _enum: {
            submit: {
                proposalOrigin: string;
                proposal: string;
                enactmentMoment: string;
            };
            place_decision_deposit: {
                index: string;
            };
            refund_decision_deposit: {
                index: string;
            };
            cancel: {
                index: string;
            };
            kill: {
                index: string;
            };
            nudge_referendum: {
                index: string;
            };
            one_fewer_deciding: {
                track: string;
            };
            refund_submission_deposit: {
                index: string;
            };
            set_metadata: {
                index: string;
                maybeHash: string;
            };
        };
    };
    /**
     * Lookup390: frame_support::traits::schedule::DispatchTime<BlockNumber>
     **/
    FrameSupportScheduleDispatchTime: {
        _enum: {
            At: string;
            After: string;
        };
    };
    /**
     * Lookup391: pallet_remark::pallet::Call<T>
     **/
    PalletRemarkCall: {
        _enum: {
            store: {
                remark: string;
            };
        };
    };
    /**
     * Lookup392: pallet_root_testing::pallet::Call<T>
     **/
    PalletRootTestingCall: {
        _enum: {
            fill_block: {
                ratio: string;
            };
            trigger_defensive: string;
        };
    };
    /**
     * Lookup393: pallet_conviction_voting::pallet::Call<T, I>
     **/
    PalletConvictionVotingCall: {
        _enum: {
            vote: {
                pollIndex: string;
                vote: string;
            };
            delegate: {
                class: string;
                to: string;
                conviction: string;
                balance: string;
            };
            undelegate: {
                class: string;
            };
            unlock: {
                class: string;
                target: string;
            };
            remove_vote: {
                class: string;
                index: string;
            };
            remove_other_vote: {
                target: string;
                class: string;
                index: string;
            };
        };
    };
    /**
     * Lookup394: pallet_conviction_voting::vote::AccountVote<Balance>
     **/
    PalletConvictionVotingVoteAccountVote: {
        _enum: {
            Standard: {
                vote: string;
                balance: string;
            };
            Split: {
                aye: string;
                nay: string;
            };
            SplitAbstain: {
                aye: string;
                nay: string;
                abstain: string;
            };
        };
    };
    /**
     * Lookup396: pallet_conviction_voting::conviction::Conviction
     **/
    PalletConvictionVotingConviction: {
        _enum: string[];
    };
    /**
     * Lookup397: pallet_whitelist::pallet::Call<T>
     **/
    PalletWhitelistCall: {
        _enum: {
            whitelist_call: {
                callHash: string;
            };
            remove_whitelisted_call: {
                callHash: string;
            };
            dispatch_whitelisted_call: {
                callHash: string;
                callEncodedLen: string;
                callWeightWitness: string;
            };
            dispatch_whitelisted_call_with_preimage: {
                call: string;
            };
        };
    };
    /**
     * Lookup399: pallet_alliance::pallet::Call<T, I>
     **/
    PalletAllianceCall: {
        _enum: {
            propose: {
                threshold: string;
                proposal: string;
                lengthBound: string;
            };
            vote: {
                proposal: string;
                index: string;
                approve: string;
            };
            __Unused2: string;
            init_members: {
                fellows: string;
                allies: string;
            };
            disband: {
                witness: string;
            };
            set_rule: {
                rule: string;
            };
            announce: {
                announcement: string;
            };
            remove_announcement: {
                announcement: string;
            };
            join_alliance: string;
            nominate_ally: {
                who: string;
            };
            elevate_ally: {
                ally: string;
            };
            give_retirement_notice: string;
            retire: string;
            kick_member: {
                who: string;
            };
            add_unscrupulous_items: {
                items: string;
            };
            remove_unscrupulous_items: {
                items: string;
            };
            close: {
                proposalHash: string;
                index: string;
                proposalWeightBound: string;
                lengthBound: string;
            };
            abdicate_fellow_status: string;
        };
    };
    /**
     * Lookup400: pallet_alliance::types::DisbandWitness
     **/
    PalletAllianceDisbandWitness: {
        fellowMembers: string;
        allyMembers: string;
    };
    /**
     * Lookup401: pallet_alliance::types::Cid
     **/
    PalletAllianceCid: {
        _alias: {
            hash_: string;
        };
        version: string;
        codec: string;
        hash_: string;
    };
    /**
     * Lookup402: pallet_alliance::types::Version
     **/
    PalletAllianceVersion: {
        _enum: string[];
    };
    /**
     * Lookup403: pallet_alliance::types::Multihash
     **/
    PalletAllianceMultihash: {
        code: string;
        digest: string;
    };
    /**
     * Lookup406: pallet_alliance::UnscrupulousItem<sp_core::crypto::AccountId32, bounded_collections::bounded_vec::BoundedVec<T, S>>
     **/
    PalletAllianceUnscrupulousItem: {
        _enum: {
            AccountId: string;
            Website: string;
        };
    };
    /**
     * Lookup408: pallet_nomination_pools::pallet::Call<T>
     **/
    PalletNominationPoolsCall: {
        _enum: {
            join: {
                amount: string;
                poolId: string;
            };
            bond_extra: {
                extra: string;
            };
            claim_payout: string;
            unbond: {
                memberAccount: string;
                unbondingPoints: string;
            };
            pool_withdraw_unbonded: {
                poolId: string;
                numSlashingSpans: string;
            };
            withdraw_unbonded: {
                memberAccount: string;
                numSlashingSpans: string;
            };
            create: {
                amount: string;
                root: string;
                nominator: string;
                bouncer: string;
            };
            create_with_pool_id: {
                amount: string;
                root: string;
                nominator: string;
                bouncer: string;
                poolId: string;
            };
            nominate: {
                poolId: string;
                validators: string;
            };
            set_state: {
                poolId: string;
                state: string;
            };
            set_metadata: {
                poolId: string;
                metadata: string;
            };
            set_configs: {
                minJoinBond: string;
                minCreateBond: string;
                maxPools: string;
                maxMembers: string;
                maxMembersPerPool: string;
                globalMaxCommission: string;
            };
            update_roles: {
                poolId: string;
                newRoot: string;
                newNominator: string;
                newBouncer: string;
            };
            chill: {
                poolId: string;
            };
            bond_extra_other: {
                member: string;
                extra: string;
            };
            set_claim_permission: {
                permission: string;
            };
            claim_payout_other: {
                other: string;
            };
            set_commission: {
                poolId: string;
                newCommission: string;
            };
            set_commission_max: {
                poolId: string;
                maxCommission: string;
            };
            set_commission_change_rate: {
                poolId: string;
                changeRate: string;
            };
            claim_commission: {
                poolId: string;
            };
            adjust_pool_deposit: {
                poolId: string;
            };
            set_commission_claim_permission: {
                poolId: string;
                permission: string;
            };
            apply_slash: {
                memberAccount: string;
            };
            migrate_delegation: {
                memberAccount: string;
            };
            migrate_pool_to_delegate_stake: {
                poolId: string;
            };
        };
    };
    /**
     * Lookup409: pallet_nomination_pools::BondExtra<Balance>
     **/
    PalletNominationPoolsBondExtra: {
        _enum: {
            FreeBalance: string;
            Rewards: string;
        };
    };
    /**
     * Lookup410: pallet_nomination_pools::PoolState
     **/
    PalletNominationPoolsPoolState: {
        _enum: string[];
    };
    /**
     * Lookup411: pallet_nomination_pools::ConfigOp<T>
     **/
    PalletNominationPoolsConfigOpU128: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup412: pallet_nomination_pools::ConfigOp<T>
     **/
    PalletNominationPoolsConfigOpU32: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup413: pallet_nomination_pools::ConfigOp<sp_arithmetic::per_things::Perbill>
     **/
    PalletNominationPoolsConfigOpPerbill: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup414: pallet_nomination_pools::ConfigOp<sp_core::crypto::AccountId32>
     **/
    PalletNominationPoolsConfigOpAccountId32: {
        _enum: {
            Noop: string;
            Set: string;
            Remove: string;
        };
    };
    /**
     * Lookup415: pallet_nomination_pools::ClaimPermission
     **/
    PalletNominationPoolsClaimPermission: {
        _enum: string[];
    };
    /**
     * Lookup418: pallet_nomination_pools::CommissionChangeRate<BlockNumber>
     **/
    PalletNominationPoolsCommissionChangeRate: {
        maxIncrease: string;
        minDelay: string;
    };
    /**
     * Lookup420: pallet_nomination_pools::CommissionClaimPermission<sp_core::crypto::AccountId32>
     **/
    PalletNominationPoolsCommissionClaimPermission: {
        _enum: {
            Permissionless: string;
            Account: string;
        };
    };
    /**
     * Lookup422: pallet_ranked_collective::pallet::Call<T, I>
     **/
    PalletRankedCollectiveCall: {
        _enum: {
            add_member: {
                who: string;
            };
            promote_member: {
                who: string;
            };
            demote_member: {
                who: string;
            };
            remove_member: {
                who: string;
                minRank: string;
            };
            vote: {
                poll: string;
                aye: string;
            };
            cleanup_poll: {
                pollIndex: string;
                max: string;
            };
            exchange_member: {
                who: string;
                newWho: string;
            };
        };
    };
    /**
     * Lookup423: pallet_asset_conversion::pallet::Call<T>
     **/
    PalletAssetConversionCall: {
        _enum: {
            create_pool: {
                asset1: string;
                asset2: string;
            };
            add_liquidity: {
                asset1: string;
                asset2: string;
                amount1Desired: string;
                amount2Desired: string;
                amount1Min: string;
                amount2Min: string;
                mintTo: string;
            };
            remove_liquidity: {
                asset1: string;
                asset2: string;
                lpTokenBurn: string;
                amount1MinReceive: string;
                amount2MinReceive: string;
                withdrawTo: string;
            };
            swap_exact_tokens_for_tokens: {
                path: string;
                amountIn: string;
                amountOutMin: string;
                sendTo: string;
                keepAlive: string;
            };
            swap_tokens_for_exact_tokens: {
                path: string;
                amountOut: string;
                amountInMax: string;
                sendTo: string;
                keepAlive: string;
            };
            touch: {
                asset1: string;
                asset2: string;
            };
        };
    };
    /**
     * Lookup425: pallet_fast_unstake::pallet::Call<T>
     **/
    PalletFastUnstakeCall: {
        _enum: {
            register_fast_unstake: string;
            deregister: string;
            control: {
                erasToCheck: string;
            };
        };
    };
    /**
     * Lookup426: pallet_message_queue::pallet::Call<T>
     **/
    PalletMessageQueueCall: {
        _enum: {
            reap_page: {
                messageOrigin: string;
                pageIndex: string;
            };
            execute_overweight: {
                messageOrigin: string;
                page: string;
                index: string;
                weightLimit: string;
            };
        };
    };
    /**
     * Lookup427: frame_benchmarking_pallet_pov::pallet::Call<T>
     **/
    FrameBenchmarkingPalletPovCall: {
        _enum: string[];
    };
    /**
     * Lookup428: pallet_tx_pause::pallet::Call<T>
     **/
    PalletTxPauseCall: {
        _enum: {
            pause: {
                fullName: string;
            };
            unpause: {
                ident: string;
            };
        };
    };
    /**
     * Lookup430: pallet_safe_mode::pallet::Call<T>
     **/
    PalletSafeModeCall: {
        _enum: {
            enter: string;
            force_enter: string;
            extend: string;
            force_extend: string;
            force_exit: string;
            force_slash_deposit: {
                account: string;
                block: string;
            };
            release_deposit: {
                account: string;
                block: string;
            };
            force_release_deposit: {
                account: string;
                block: string;
            };
        };
    };
    /**
     * Lookup431: pallet_migrations::pallet::Call<T>
     **/
    PalletMigrationsCall: {
        _enum: {
            force_set_cursor: {
                cursor: string;
            };
            force_set_active_cursor: {
                index: string;
                innerCursor: string;
                startedAt: string;
            };
            force_onboard_mbms: string;
            clear_historic: {
                selector: string;
            };
        };
    };
    /**
     * Lookup433: pallet_migrations::MigrationCursor<bounded_collections::bounded_vec::BoundedVec<T, S>, BlockNumber>
     **/
    PalletMigrationsMigrationCursor: {
        _enum: {
            Active: string;
            Stuck: string;
        };
    };
    /**
     * Lookup435: pallet_migrations::ActiveCursor<bounded_collections::bounded_vec::BoundedVec<T, S>, BlockNumber>
     **/
    PalletMigrationsActiveCursor: {
        index: string;
        innerCursor: string;
        startedAt: string;
    };
    /**
     * Lookup437: pallet_migrations::HistoricCleanupSelector<bounded_collections::bounded_vec::BoundedVec<T, S>>
     **/
    PalletMigrationsHistoricCleanupSelector: {
        _enum: {
            Specific: string;
            Wildcard: {
                limit: string;
                previousCursor: string;
            };
        };
    };
    /**
     * Lookup440: pallet_broker::pallet::Call<T>
     **/
    PalletBrokerCall: {
        _enum: {
            configure: {
                config: string;
            };
            reserve: {
                workload: string;
            };
            unreserve: {
                itemIndex: string;
            };
            set_lease: {
                task: string;
                until: string;
            };
            start_sales: {
                endPrice: string;
                extraCores: string;
            };
            purchase: {
                priceLimit: string;
            };
            renew: {
                core: string;
            };
            transfer: {
                regionId: string;
                newOwner: string;
            };
            partition: {
                regionId: string;
                pivot: string;
            };
            interlace: {
                regionId: string;
                pivot: string;
            };
            assign: {
                regionId: string;
                task: string;
                finality: string;
            };
            pool: {
                regionId: string;
                payee: string;
                finality: string;
            };
            claim_revenue: {
                regionId: string;
                maxTimeslices: string;
            };
            purchase_credit: {
                amount: string;
                beneficiary: string;
            };
            drop_region: {
                regionId: string;
            };
            drop_contribution: {
                regionId: string;
            };
            drop_history: {
                when: string;
            };
            drop_renewal: {
                core: string;
                when: string;
            };
            request_core_count: {
                coreCount: string;
            };
            notify_core_count: {
                coreCount: string;
            };
            notify_revenue: {
                revenue: string;
            };
            enable_auto_renew: {
                core: string;
                task: string;
                workloadEndHint: string;
            };
            disable_auto_renew: {
                core: string;
                task: string;
            };
            force_reserve: {
                workload: string;
                core: string;
            };
            remove_lease: {
                task: string;
            };
            __Unused25: string;
            remove_assignment: {
                regionId: string;
            };
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
            __Unused98: string;
            swap_leases: {
                id: string;
                other: string;
            };
        };
    };
    /**
     * Lookup441: pallet_broker::types::ConfigRecord<RelayBlockNumber>
     **/
    PalletBrokerConfigRecord: {
        advanceNotice: string;
        interludeLength: string;
        leadinLength: string;
        regionLength: string;
        idealBulkProportion: string;
        limitCoresOffered: string;
        renewalBump: string;
        contributionTimeout: string;
    };
    /**
     * Lookup443: pallet_broker::types::ScheduleItem
     **/
    PalletBrokerScheduleItem: {
        mask: string;
        assignment: string;
    };
    /**
     * Lookup444: pallet_broker::core_mask::CoreMask
     **/
    PalletBrokerCoreMask: string;
    /**
     * Lookup445: pallet_broker::coretime_interface::CoreAssignment
     **/
    PalletBrokerCoretimeInterfaceCoreAssignment: {
        _enum: {
            Idle: string;
            Pool: string;
            Task: string;
        };
    };
    /**
     * Lookup447: pallet_broker::types::RegionId
     **/
    PalletBrokerRegionId: {
        begin: string;
        core: string;
        mask: string;
    };
    /**
     * Lookup448: pallet_broker::types::Finality
     **/
    PalletBrokerFinality: {
        _enum: string[];
    };
    /**
     * Lookup449: pallet_broker::types::OnDemandRevenueRecord<RelayBlockNumber, RelayBalance>
     **/
    PalletBrokerOnDemandRevenueRecord: {
        until: string;
        amount: string;
    };
    /**
     * Lookup450: pallet_mixnet::pallet::Call<T>
     **/
    PalletMixnetCall: {
        _enum: {
            register: {
                registration: string;
                signature: string;
            };
        };
    };
    /**
     * Lookup451: pallet_mixnet::Registration<BlockNumber, pallet_mixnet::BoundedMixnode<bounded_collections::bounded_vec::BoundedVec<bounded_collections::bounded_vec::BoundedVec<T, S>, S>>>
     **/
    PalletMixnetRegistration: {
        blockNumber: string;
        sessionIndex: string;
        authorityIndex: string;
        mixnode: string;
    };
    /**
     * Lookup452: pallet_mixnet::BoundedMixnode<bounded_collections::bounded_vec::BoundedVec<bounded_collections::bounded_vec::BoundedVec<T, S>, S>>
     **/
    PalletMixnetBoundedMixnode: {
        kxPublic: string;
        peerId: string;
        externalAddresses: string;
    };
    /**
     * Lookup455: sp_mixnet::types::app::Signature
     **/
    SpMixnetAppSignature: string;
    /**
     * Lookup456: pallet_parameters::pallet::Call<T>
     **/
    PalletParametersCall: {
        _enum: {
            set_parameter: {
                keyValue: string;
            };
        };
    };
    /**
     * Lookup457: kitchensink_runtime::RuntimeParameters
     **/
    KitchensinkRuntimeRuntimeParameters: {
        _enum: {
            Storage: string;
            Referenda: string;
        };
    };
    /**
     * Lookup458: kitchensink_runtime::dynamic_params::storage::Parameters
     **/
    KitchensinkRuntimeDynamicParamsStorageParameters: {
        _enum: {
            BaseDeposit: string;
            ByteDeposit: string;
        };
    };
    /**
     * Lookup459: kitchensink_runtime::dynamic_params::storage::BaseDeposit
     **/
    KitchensinkRuntimeDynamicParamsStorageBaseDeposit: string;
    /**
     * Lookup460: kitchensink_runtime::dynamic_params::storage::ByteDeposit
     **/
    KitchensinkRuntimeDynamicParamsStorageByteDeposit: string;
    /**
     * Lookup461: kitchensink_runtime::dynamic_params::referenda::Parameters
     **/
    KitchensinkRuntimeDynamicParamsReferendaParameters: {
        _enum: {
            Tracks: string;
            Origins: string;
        };
    };
    /**
     * Lookup462: kitchensink_runtime::dynamic_params::referenda::Tracks
     **/
    KitchensinkRuntimeDynamicParamsReferendaTracks: string;
    /**
     * Lookup465: pallet_referenda::types::Track<Id, Balance, Moment>
     **/
    PalletReferendaTrack: {
        id: string;
        info: {
            name: string;
            maxDeciding: string;
            decisionDeposit: string;
            preparePeriod: string;
            decisionPeriod: string;
            confirmPeriod: string;
            minEnactmentPeriod: string;
            minApproval: string;
            minSupport: string;
        };
    };
    /**
     * Lookup467: pallet_referenda::types::Curve
     **/
    PalletReferendaCurve: {
        _enum: {
            LinearDecreasing: {
                length: string;
                floor: string;
                ceil: string;
            };
            SteppedDecreasing: {
                begin: string;
                end: string;
                step: string;
                period: string;
            };
            Reciprocal: {
                factor: string;
                xOffset: string;
                yOffset: string;
            };
        };
    };
    /**
     * Lookup471: kitchensink_runtime::dynamic_params::referenda::Origins
     **/
    KitchensinkRuntimeDynamicParamsReferendaOrigins: string;
    /**
     * Lookup476: pallet_asset_conversion_ops::pallet::Call<T>
     **/
    PalletAssetConversionOpsCall: {
        _enum: {
            migrate_to_new_account: {
                asset1: string;
                asset2: string;
            };
        };
    };
    /**
     * Lookup477: pallet_revive::pallet::Call<T>
     **/
    PalletReviveCall: {
        _enum: {
            eth_transact: {
                payload: string;
            };
            call: {
                dest: string;
                value: string;
                gasLimit: string;
                storageDepositLimit: string;
                data: string;
            };
            instantiate: {
                value: string;
                gasLimit: string;
                storageDepositLimit: string;
                codeHash: string;
                data: string;
                salt: string;
            };
            instantiate_with_code: {
                value: string;
                gasLimit: string;
                storageDepositLimit: string;
                code: string;
                data: string;
                salt: string;
            };
            upload_code: {
                code: string;
                storageDepositLimit: string;
            };
            remove_code: {
                codeHash: string;
            };
            set_code: {
                dest: string;
                codeHash: string;
            };
            map_account: string;
            unmap_account: string;
            dispatch_as_fallback_account: {
                call: string;
            };
        };
    };
    /**
     * Lookup479: pallet_asset_rewards::pallet::Call<T>
     **/
    PalletAssetRewardsCall: {
        _enum: {
            create_pool: {
                stakedAssetId: string;
                rewardAssetId: string;
                rewardRatePerBlock: string;
                expiry: string;
                admin: string;
            };
            stake: {
                poolId: string;
                amount: string;
            };
            unstake: {
                poolId: string;
                amount: string;
                staker: string;
            };
            harvest_rewards: {
                poolId: string;
                staker: string;
            };
            set_pool_reward_rate_per_block: {
                poolId: string;
                newRewardRatePerBlock: string;
            };
            set_pool_admin: {
                poolId: string;
                newAdmin: string;
            };
            set_pool_expiry_block: {
                poolId: string;
                newExpiry: string;
            };
            deposit_reward_tokens: {
                poolId: string;
                amount: string;
            };
            cleanup_pool: {
                poolId: string;
            };
        };
    };
    /**
     * Lookup480: pallet_meta_tx::pallet::Call<T>
     **/
    PalletMetaTxCall: {
        _enum: {
            dispatch: {
                metaTx: string;
            };
        };
    };
    /**
     * Lookup481: pallet_meta_tx::MetaTx<kitchensink_runtime::RuntimeCall, Extension>
     **/
    PalletMetaTxMetaTx: {
        call: string;
        extensionVersion: string;
        extension: string;
    };
    /**
     * Lookup483: pallet_verify_signature::extension::VerifySignature<T>
     **/
    PalletVerifySignatureExtensionVerifySignature: {
        _enum: {
            Signed: {
                signature: string;
                account: string;
            };
            Disabled: string;
        };
    };
    /**
     * Lookup484: pallet_meta_tx::extension::MetaTxMarker<T>
     **/
    PalletMetaTxExtensionMetaTxMarker: string;
    /**
     * Lookup485: frame_system::extensions::check_non_zero_sender::CheckNonZeroSender<T>
     **/
    FrameSystemExtensionsCheckNonZeroSender: string;
    /**
     * Lookup486: frame_system::extensions::check_spec_version::CheckSpecVersion<T>
     **/
    FrameSystemExtensionsCheckSpecVersion: string;
    /**
     * Lookup487: frame_system::extensions::check_tx_version::CheckTxVersion<T>
     **/
    FrameSystemExtensionsCheckTxVersion: string;
    /**
     * Lookup488: frame_system::extensions::check_genesis::CheckGenesis<T>
     **/
    FrameSystemExtensionsCheckGenesis: string;
    /**
     * Lookup491: frame_system::extensions::check_nonce::CheckNonce<T>
     **/
    FrameSystemExtensionsCheckNonce: string;
    /**
     * Lookup492: frame_metadata_hash_extension::CheckMetadataHash<T>
     **/
    FrameMetadataHashExtensionCheckMetadataHash: {
        mode: string;
    };
    /**
     * Lookup493: frame_metadata_hash_extension::Mode
     **/
    FrameMetadataHashExtensionMode: {
        _enum: string[];
    };
    /**
     * Lookup494: sp_runtime::traits::BlakeTwo256
     **/
    SpRuntimeBlakeTwo256: string;
    /**
     * Lookup495: pallet_conviction_voting::types::Tally<Votes, Total>
     **/
    PalletConvictionVotingTally: {
        ayes: string;
        nays: string;
        support: string;
    };
    /**
     * Lookup496: pallet_remark::pallet::Event<T>
     **/
    PalletRemarkEvent: {
        _enum: {
            Stored: {
                sender: string;
                contentHash: string;
            };
        };
    };
    /**
     * Lookup497: pallet_root_testing::pallet::Event<T>
     **/
    PalletRootTestingEvent: {
        _enum: string[];
    };
    /**
     * Lookup498: pallet_conviction_voting::pallet::Event<T, I>
     **/
    PalletConvictionVotingEvent: {
        _enum: {
            Delegated: string;
            Undelegated: string;
            Voted: {
                who: string;
                vote: string;
            };
            VoteRemoved: {
                who: string;
                vote: string;
            };
            VoteUnlocked: {
                who: string;
                class: string;
            };
        };
    };
    /**
     * Lookup499: pallet_whitelist::pallet::Event<T>
     **/
    PalletWhitelistEvent: {
        _enum: {
            CallWhitelisted: {
                callHash: string;
            };
            WhitelistedCallRemoved: {
                callHash: string;
            };
            WhitelistedCallDispatched: {
                callHash: string;
                result: string;
            };
        };
    };
    /**
     * Lookup501: frame_support::dispatch::PostDispatchInfo
     **/
    FrameSupportDispatchPostDispatchInfo: {
        actualWeight: string;
        paysFee: string;
    };
    /**
     * Lookup503: sp_runtime::DispatchErrorWithPostInfo<frame_support::dispatch::PostDispatchInfo>
     **/
    SpRuntimeDispatchErrorWithPostInfo: {
        postInfo: string;
        error: string;
    };
    /**
     * Lookup505: pallet_alliance::pallet::Event<T, I>
     **/
    PalletAllianceEvent: {
        _enum: {
            NewRuleSet: {
                rule: string;
            };
            Announced: {
                announcement: string;
            };
            AnnouncementRemoved: {
                announcement: string;
            };
            MembersInitialized: {
                fellows: string;
                allies: string;
            };
            NewAllyJoined: {
                ally: string;
                nominator: string;
                reserved: string;
            };
            AllyElevated: {
                ally: string;
            };
            MemberRetirementPeriodStarted: {
                member: string;
            };
            MemberRetired: {
                member: string;
                unreserved: string;
            };
            MemberKicked: {
                member: string;
                slashed: string;
            };
            UnscrupulousItemAdded: {
                items: string;
            };
            UnscrupulousItemRemoved: {
                items: string;
            };
            AllianceDisbanded: {
                fellowMembers: string;
                allyMembers: string;
                unreserved: string;
            };
            FellowAbdicated: {
                fellow: string;
            };
        };
    };
    /**
     * Lookup506: pallet_nomination_pools::pallet::Event<T>
     **/
    PalletNominationPoolsEvent: {
        _enum: {
            Created: {
                depositor: string;
                poolId: string;
            };
            Bonded: {
                member: string;
                poolId: string;
                bonded: string;
                joined: string;
            };
            PaidOut: {
                member: string;
                poolId: string;
                payout: string;
            };
            Unbonded: {
                member: string;
                poolId: string;
                balance: string;
                points: string;
                era: string;
            };
            Withdrawn: {
                member: string;
                poolId: string;
                balance: string;
                points: string;
            };
            Destroyed: {
                poolId: string;
            };
            StateChanged: {
                poolId: string;
                newState: string;
            };
            MemberRemoved: {
                poolId: string;
                member: string;
                releasedBalance: string;
            };
            RolesUpdated: {
                root: string;
                bouncer: string;
                nominator: string;
            };
            PoolSlashed: {
                poolId: string;
                balance: string;
            };
            UnbondingPoolSlashed: {
                poolId: string;
                era: string;
                balance: string;
            };
            PoolCommissionUpdated: {
                poolId: string;
                current: string;
            };
            PoolMaxCommissionUpdated: {
                poolId: string;
                maxCommission: string;
            };
            PoolCommissionChangeRateUpdated: {
                poolId: string;
                changeRate: string;
            };
            PoolCommissionClaimPermissionUpdated: {
                poolId: string;
                permission: string;
            };
            PoolCommissionClaimed: {
                poolId: string;
                commission: string;
            };
            MinBalanceDeficitAdjusted: {
                poolId: string;
                amount: string;
            };
            MinBalanceExcessAdjusted: {
                poolId: string;
                amount: string;
            };
            MemberClaimPermissionUpdated: {
                member: string;
                permission: string;
            };
            MetadataUpdated: {
                poolId: string;
                caller: string;
            };
            PoolNominationMade: {
                poolId: string;
                caller: string;
            };
            PoolNominatorChilled: {
                poolId: string;
                caller: string;
            };
            GlobalParamsUpdated: {
                minJoinBond: string;
                minCreateBond: string;
                maxPools: string;
                maxMembers: string;
                maxMembersPerPool: string;
                globalMaxCommission: string;
            };
        };
    };
    /**
     * Lookup509: pallet_ranked_collective::Tally<T, I, M>
     **/
    PalletRankedCollectiveTally: {
        bareAyes: string;
        ayes: string;
        nays: string;
    };
    /**
     * Lookup510: pallet_ranked_collective::pallet::Event<T, I>
     **/
    PalletRankedCollectiveEvent: {
        _enum: {
            MemberAdded: {
                who: string;
            };
            RankChanged: {
                who: string;
                rank: string;
            };
            MemberRemoved: {
                who: string;
                rank: string;
            };
            Voted: {
                who: string;
                poll: string;
                vote: string;
                tally: string;
            };
            MemberExchanged: {
                who: string;
                newWho: string;
            };
        };
    };
    /**
     * Lookup511: pallet_ranked_collective::VoteRecord
     **/
    PalletRankedCollectiveVoteRecord: {
        _enum: {
            Aye: string;
            Nay: string;
        };
    };
    /**
     * Lookup512: pallet_asset_conversion::pallet::Event<T>
     **/
    PalletAssetConversionEvent: {
        _enum: {
            PoolCreated: {
                creator: string;
                poolId: string;
                poolAccount: string;
                lpToken: string;
            };
            LiquidityAdded: {
                who: string;
                mintTo: string;
                poolId: string;
                amount1Provided: string;
                amount2Provided: string;
                lpToken: string;
                lpTokenMinted: string;
            };
            LiquidityRemoved: {
                who: string;
                withdrawTo: string;
                poolId: string;
                amount1: string;
                amount2: string;
                lpToken: string;
                lpTokenBurned: string;
                withdrawalFee: string;
            };
            SwapExecuted: {
                who: string;
                sendTo: string;
                amountIn: string;
                amountOut: string;
                path: string;
            };
            SwapCreditExecuted: {
                amountIn: string;
                amountOut: string;
                path: string;
            };
            Touched: {
                poolId: string;
                who: string;
            };
        };
    };
    /**
     * Lookup517: pallet_fast_unstake::pallet::Event<T>
     **/
    PalletFastUnstakeEvent: {
        _enum: {
            Unstaked: {
                stash: string;
                result: string;
            };
            Slashed: {
                stash: string;
                amount: string;
            };
            BatchChecked: {
                eras: string;
            };
            BatchFinished: {
                _alias: {
                    size_: string;
                };
                size_: string;
            };
            InternalError: string;
        };
    };
    /**
     * Lookup518: pallet_message_queue::pallet::Event<T>
     **/
    PalletMessageQueueEvent: {
        _enum: {
            ProcessingFailed: {
                id: string;
                origin: string;
                error: string;
            };
            Processed: {
                id: string;
                origin: string;
                weightUsed: string;
                success: string;
            };
            OverweightEnqueued: {
                id: string;
                origin: string;
                pageIndex: string;
                messageIndex: string;
            };
            PageReaped: {
                origin: string;
                index: string;
            };
        };
    };
    /**
     * Lookup519: frame_support::traits::messages::ProcessMessageError
     **/
    FrameSupportMessagesProcessMessageError: {
        _enum: {
            BadFormat: string;
            Corrupt: string;
            Unsupported: string;
            Overweight: string;
            Yield: string;
            StackLimitReached: string;
        };
    };
    /**
     * Lookup520: frame_benchmarking_pallet_pov::pallet::Event<T>
     **/
    FrameBenchmarkingPalletPovEvent: {
        _enum: string[];
    };
    /**
     * Lookup521: pallet_tx_pause::pallet::Event<T>
     **/
    PalletTxPauseEvent: {
        _enum: {
            CallPaused: {
                fullName: string;
            };
            CallUnpaused: {
                fullName: string;
            };
        };
    };
    /**
     * Lookup522: pallet_safe_mode::pallet::Event<T>
     **/
    PalletSafeModeEvent: {
        _enum: {
            Entered: {
                until: string;
            };
            Extended: {
                until: string;
            };
            Exited: {
                reason: string;
            };
            DepositPlaced: {
                account: string;
                amount: string;
            };
            DepositReleased: {
                account: string;
                amount: string;
            };
            DepositSlashed: {
                account: string;
                amount: string;
            };
            CannotDeposit: string;
            CannotRelease: string;
        };
    };
    /**
     * Lookup523: pallet_safe_mode::pallet::ExitReason
     **/
    PalletSafeModeExitReason: {
        _enum: string[];
    };
    /**
     * Lookup524: pallet_statement::pallet::Event<T>
     **/
    PalletStatementEvent: {
        _enum: {
            NewStatement: {
                account: string;
                statement: string;
            };
        };
    };
    /**
     * Lookup525: sp_statement_store::Statement
     **/
    SpStatementStoreStatement: {
        proof: string;
        decryptionKey: string;
        channel: string;
        priority: string;
        numTopics: string;
        topics: string;
        data: string;
    };
    /**
     * Lookup527: sp_statement_store::Proof
     **/
    SpStatementStoreProof: {
        _enum: {
            Sr25519: {
                signature: string;
                signer: string;
            };
            Ed25519: {
                signature: string;
                signer: string;
            };
            Secp256k1Ecdsa: {
                signature: string;
                signer: string;
            };
            OnChain: {
                who: string;
                blockHash: string;
                eventIndex: string;
            };
        };
    };
    /**
     * Lookup529: pallet_migrations::pallet::Event<T>
     **/
    PalletMigrationsEvent: {
        _enum: {
            UpgradeStarted: {
                migrations: string;
            };
            UpgradeCompleted: string;
            UpgradeFailed: string;
            MigrationSkipped: {
                index: string;
            };
            MigrationAdvanced: {
                index: string;
                took: string;
            };
            MigrationCompleted: {
                index: string;
                took: string;
            };
            MigrationFailed: {
                index: string;
                took: string;
            };
            HistoricCleared: {
                nextCursor: string;
            };
        };
    };
    /**
     * Lookup530: pallet_broker::pallet::Event<T>
     **/
    PalletBrokerEvent: {
        _enum: {
            Purchased: {
                who: string;
                regionId: string;
                price: string;
                duration: string;
            };
            Renewable: {
                core: string;
                price: string;
                begin: string;
                workload: string;
            };
            Renewed: {
                who: string;
                price: string;
                oldCore: string;
                core: string;
                begin: string;
                duration: string;
                workload: string;
            };
            Transferred: {
                regionId: string;
                duration: string;
                oldOwner: string;
                owner: string;
            };
            Partitioned: {
                oldRegionId: string;
                newRegionIds: string;
            };
            Interlaced: {
                oldRegionId: string;
                newRegionIds: string;
            };
            Assigned: {
                regionId: string;
                duration: string;
                task: string;
            };
            AssignmentRemoved: {
                regionId: string;
            };
            Pooled: {
                regionId: string;
                duration: string;
            };
            CoreCountRequested: {
                coreCount: string;
            };
            CoreCountChanged: {
                coreCount: string;
            };
            ReservationMade: {
                index: string;
                workload: string;
            };
            ReservationCancelled: {
                index: string;
                workload: string;
            };
            SaleInitialized: {
                saleStart: string;
                leadinLength: string;
                startPrice: string;
                endPrice: string;
                regionBegin: string;
                regionEnd: string;
                idealCoresSold: string;
                coresOffered: string;
            };
            Leased: {
                task: string;
                until: string;
            };
            LeaseRemoved: {
                task: string;
            };
            LeaseEnding: {
                task: string;
                when: string;
            };
            SalesStarted: {
                price: string;
                coreCount: string;
            };
            RevenueClaimBegun: {
                region: string;
                maxTimeslices: string;
            };
            RevenueClaimItem: {
                when: string;
                amount: string;
            };
            RevenueClaimPaid: {
                who: string;
                amount: string;
                next: string;
            };
            CreditPurchased: {
                who: string;
                beneficiary: string;
                amount: string;
            };
            RegionDropped: {
                regionId: string;
                duration: string;
            };
            ContributionDropped: {
                regionId: string;
            };
            HistoryInitialized: {
                when: string;
                privatePoolSize: string;
                systemPoolSize: string;
            };
            HistoryDropped: {
                when: string;
                revenue: string;
            };
            HistoryIgnored: {
                when: string;
                revenue: string;
            };
            ClaimsReady: {
                when: string;
                systemPayout: string;
                privatePayout: string;
            };
            CoreAssigned: {
                core: string;
                when: string;
                assignment: string;
            };
            PotentialRenewalDropped: {
                when: string;
                core: string;
            };
            AutoRenewalEnabled: {
                core: string;
                task: string;
            };
            AutoRenewalDisabled: {
                core: string;
                task: string;
            };
            AutoRenewalFailed: {
                core: string;
                payer: string;
            };
            AutoRenewalLimitReached: string;
        };
    };
    /**
     * Lookup535: pallet_parameters::pallet::Event<T>
     **/
    PalletParametersEvent: {
        _enum: {
            Updated: {
                key: string;
                oldValue: string;
                newValue: string;
            };
        };
    };
    /**
     * Lookup536: kitchensink_runtime::RuntimeParametersKey
     **/
    KitchensinkRuntimeRuntimeParametersKey: {
        _enum: {
            Storage: string;
            Referenda: string;
        };
    };
    /**
     * Lookup537: kitchensink_runtime::dynamic_params::storage::ParametersKey
     **/
    KitchensinkRuntimeDynamicParamsStorageParametersKey: {
        _enum: string[];
    };
    /**
     * Lookup538: kitchensink_runtime::dynamic_params::referenda::ParametersKey
     **/
    KitchensinkRuntimeDynamicParamsReferendaParametersKey: {
        _enum: string[];
    };
    /**
     * Lookup540: kitchensink_runtime::RuntimeParametersValue
     **/
    KitchensinkRuntimeRuntimeParametersValue: {
        _enum: {
            Storage: string;
            Referenda: string;
        };
    };
    /**
     * Lookup541: kitchensink_runtime::dynamic_params::storage::ParametersValue
     **/
    KitchensinkRuntimeDynamicParamsStorageParametersValue: {
        _enum: {
            BaseDeposit: string;
            ByteDeposit: string;
        };
    };
    /**
     * Lookup542: kitchensink_runtime::dynamic_params::referenda::ParametersValue
     **/
    KitchensinkRuntimeDynamicParamsReferendaParametersValue: {
        _enum: {
            Tracks: string;
            Origins: string;
        };
    };
    /**
     * Lookup543: pallet_skip_feeless_payment::pallet::Event<T>
     **/
    PalletSkipFeelessPaymentEvent: {
        _enum: {
            FeeSkipped: {
                origin: string;
            };
        };
    };
    /**
     * Lookup544: pallet_asset_conversion_ops::pallet::Event<T>
     **/
    PalletAssetConversionOpsEvent: {
        _enum: {
            MigratedToNewAccount: {
                poolId: string;
                priorAccount: string;
                newAccount: string;
            };
        };
    };
    /**
     * Lookup545: pallet_revive::pallet::Event<T>
     **/
    PalletReviveEvent: {
        _enum: {
            ContractEmitted: {
                contract: string;
                data: string;
                topics: string;
            };
        };
    };
    /**
     * Lookup546: pallet_delegated_staking::pallet::Event<T>
     **/
    PalletDelegatedStakingEvent: {
        _enum: {
            Delegated: {
                agent: string;
                delegator: string;
                amount: string;
            };
            Released: {
                agent: string;
                delegator: string;
                amount: string;
            };
            Slashed: {
                agent: string;
                delegator: string;
                amount: string;
            };
            MigratedDelegation: {
                agent: string;
                delegator: string;
                amount: string;
            };
        };
    };
    /**
     * Lookup547: pallet_asset_rewards::pallet::Event<T>
     **/
    PalletAssetRewardsEvent: {
        _enum: {
            Staked: {
                staker: string;
                poolId: string;
                amount: string;
            };
            Unstaked: {
                caller: string;
                staker: string;
                poolId: string;
                amount: string;
            };
            RewardsHarvested: {
                caller: string;
                staker: string;
                poolId: string;
                amount: string;
            };
            PoolCreated: {
                creator: string;
                poolId: string;
                stakedAssetId: string;
                rewardAssetId: string;
                rewardRatePerBlock: string;
                expiryBlock: string;
                admin: string;
            };
            PoolRewardRateModified: {
                poolId: string;
                newRewardRatePerBlock: string;
            };
            PoolAdminModified: {
                poolId: string;
                newAdmin: string;
            };
            PoolExpiryBlockModified: {
                poolId: string;
                newExpiryBlock: string;
            };
            PoolCleanedUp: {
                poolId: string;
            };
        };
    };
    /**
     * Lookup548: pallet_assets_freezer::pallet::Event<T, I>
     **/
    PalletAssetsFreezerEvent: {
        _enum: {
            Frozen: {
                who: string;
                assetId: string;
                amount: string;
            };
            Thawed: {
                who: string;
                assetId: string;
                amount: string;
            };
        };
    };
    /**
     * Lookup549: pallet_meta_tx::pallet::Event<T>
     **/
    PalletMetaTxEvent: {
        _enum: {
            Dispatched: {
                result: string;
            };
        };
    };
    /**
     * Lookup550: frame_system::Phase
     **/
    FrameSystemPhase: {
        _enum: {
            ApplyExtrinsic: string;
            Finalization: string;
            Initialization: string;
        };
    };
    /**
     * Lookup552: frame_system::LastRuntimeUpgradeInfo
     **/
    FrameSystemLastRuntimeUpgradeInfo: {
        specVersion: string;
        specName: string;
    };
    /**
     * Lookup555: frame_system::CodeUpgradeAuthorization<T>
     **/
    FrameSystemCodeUpgradeAuthorization: {
        codeHash: string;
        checkVersion: string;
    };
    /**
     * Lookup556: frame_system::limits::BlockWeights
     **/
    FrameSystemLimitsBlockWeights: {
        baseBlock: string;
        maxBlock: string;
        perClass: string;
    };
    /**
     * Lookup557: frame_support::dispatch::PerDispatchClass<frame_system::limits::WeightsPerClass>
     **/
    FrameSupportDispatchPerDispatchClassWeightsPerClass: {
        normal: string;
        operational: string;
        mandatory: string;
    };
    /**
     * Lookup558: frame_system::limits::WeightsPerClass
     **/
    FrameSystemLimitsWeightsPerClass: {
        baseExtrinsic: string;
        maxExtrinsic: string;
        maxTotal: string;
        reserved: string;
    };
    /**
     * Lookup559: frame_system::limits::BlockLength
     **/
    FrameSystemLimitsBlockLength: {
        max: string;
    };
    /**
     * Lookup560: frame_support::dispatch::PerDispatchClass<T>
     **/
    FrameSupportDispatchPerDispatchClassU32: {
        normal: string;
        operational: string;
        mandatory: string;
    };
    /**
     * Lookup561: sp_weights::RuntimeDbWeight
     **/
    SpWeightsRuntimeDbWeight: {
        read: string;
        write: string;
    };
    /**
     * Lookup562: sp_version::RuntimeVersion
     **/
    SpVersionRuntimeVersion: {
        specName: string;
        implName: string;
        authoringVersion: string;
        specVersion: string;
        implVersion: string;
        apis: string;
        transactionVersion: string;
        systemVersion: string;
    };
    /**
     * Lookup566: frame_system::pallet::Error<T>
     **/
    FrameSystemError: {
        _enum: string[];
    };
    /**
     * Lookup567: pallet_utility::pallet::Error<T>
     **/
    PalletUtilityError: {
        _enum: string[];
    };
    /**
     * Lookup574: sp_consensus_babe::digests::PreDigest
     **/
    SpConsensusBabeDigestsPreDigest: {
        _enum: {
            __Unused0: string;
            Primary: string;
            SecondaryPlain: string;
            SecondaryVRF: string;
        };
    };
    /**
     * Lookup575: sp_consensus_babe::digests::PrimaryPreDigest
     **/
    SpConsensusBabeDigestsPrimaryPreDigest: {
        authorityIndex: string;
        slot: string;
        vrfSignature: string;
    };
    /**
     * Lookup576: sp_core::sr25519::vrf::VrfSignature
     **/
    SpCoreSr25519VrfVrfSignature: {
        preOutput: string;
        proof: string;
    };
    /**
     * Lookup577: sp_consensus_babe::digests::SecondaryPlainPreDigest
     **/
    SpConsensusBabeDigestsSecondaryPlainPreDigest: {
        authorityIndex: string;
        slot: string;
    };
    /**
     * Lookup578: sp_consensus_babe::digests::SecondaryVRFPreDigest
     **/
    SpConsensusBabeDigestsSecondaryVRFPreDigest: {
        authorityIndex: string;
        slot: string;
        vrfSignature: string;
    };
    /**
     * Lookup579: sp_consensus_babe::BabeEpochConfiguration
     **/
    SpConsensusBabeBabeEpochConfiguration: {
        c: string;
        allowedSlots: string;
    };
    /**
     * Lookup583: pallet_babe::pallet::Error<T>
     **/
    PalletBabeError: {
        _enum: string[];
    };
    /**
     * Lookup585: pallet_indices::pallet::Error<T>
     **/
    PalletIndicesError: {
        _enum: string[];
    };
    /**
     * Lookup587: pallet_balances::types::BalanceLock<Balance>
     **/
    PalletBalancesBalanceLock: {
        id: string;
        amount: string;
        reasons: string;
    };
    /**
     * Lookup588: pallet_balances::types::Reasons
     **/
    PalletBalancesReasons: {
        _enum: string[];
    };
    /**
     * Lookup591: pallet_balances::types::ReserveData<ReserveIdentifier, Balance>
     **/
    PalletBalancesReserveData: {
        id: string;
        amount: string;
    };
    /**
     * Lookup594: frame_support::traits::tokens::misc::IdAmount<kitchensink_runtime::RuntimeHoldReason, Balance>
     **/
    FrameSupportTokensMiscIdAmountRuntimeHoldReason: {
        id: string;
        amount: string;
    };
    /**
     * Lookup595: kitchensink_runtime::RuntimeHoldReason
     **/
    KitchensinkRuntimeRuntimeHoldReason: {
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
            Staking: string;
            __Unused12: string;
            __Unused13: string;
            Council: string;
            TechnicalCommittee: string;
            __Unused16: string;
            __Unused17: string;
            __Unused18: string;
            __Unused19: string;
            __Unused20: string;
            Contracts: string;
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
            Preimage: string;
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
            Nis: string;
            __Unused46: string;
            __Unused47: string;
            NftFractionalization: string;
            __Unused49: string;
            __Unused50: string;
            TransactionStorage: string;
            __Unused52: string;
            StateTrieMigration: string;
            __Unused54: string;
            __Unused55: string;
            __Unused56: string;
            __Unused57: string;
            __Unused58: string;
            __Unused59: string;
            AllianceMotion: string;
            __Unused61: string;
            __Unused62: string;
            __Unused63: string;
            __Unused64: string;
            __Unused65: string;
            __Unused66: string;
            __Unused67: string;
            __Unused68: string;
            __Unused69: string;
            SafeMode: string;
            __Unused71: string;
            __Unused72: string;
            __Unused73: string;
            __Unused74: string;
            __Unused75: string;
            __Unused76: string;
            __Unused77: string;
            __Unused78: string;
            __Unused79: string;
            Revive: string;
            __Unused81: string;
            DelegatedStaking: string;
            AssetRewards: string;
        };
    };
    /**
     * Lookup596: pallet_staking::pallet::pallet::HoldReason
     **/
    PalletStakingPalletHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup597: pallet_collective::pallet::HoldReason<I>
     **/
    PalletCollectiveHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup599: pallet_contracts::pallet::HoldReason
     **/
    PalletContractsHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup600: pallet_preimage::pallet::HoldReason
     **/
    PalletPreimageHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup601: pallet_nis::pallet::HoldReason
     **/
    PalletNisHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup602: pallet_nft_fractionalization::pallet::HoldReason
     **/
    PalletNftFractionalizationHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup603: pallet_transaction_storage::pallet::HoldReason
     **/
    PalletTransactionStorageHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup604: pallet_state_trie_migration::pallet::HoldReason
     **/
    PalletStateTrieMigrationHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup606: pallet_safe_mode::pallet::HoldReason
     **/
    PalletSafeModeHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup607: pallet_revive::pallet::HoldReason
     **/
    PalletReviveHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup608: pallet_delegated_staking::pallet::HoldReason
     **/
    PalletDelegatedStakingHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup609: pallet_asset_rewards::pallet::HoldReason
     **/
    PalletAssetRewardsHoldReason: {
        _enum: string[];
    };
    /**
     * Lookup612: frame_support::traits::tokens::misc::IdAmount<kitchensink_runtime::RuntimeFreezeReason, Balance>
     **/
    FrameSupportTokensMiscIdAmountRuntimeFreezeReason: {
        id: string;
        amount: string;
    };
    /**
     * Lookup613: kitchensink_runtime::RuntimeFreezeReason
     **/
    KitchensinkRuntimeRuntimeFreezeReason: {
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
            NominationPools: string;
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
            AssetRewards: string;
        };
    };
    /**
     * Lookup614: pallet_nomination_pools::pallet::FreezeReason
     **/
    PalletNominationPoolsFreezeReason: {
        _enum: string[];
    };
    /**
     * Lookup615: pallet_asset_rewards::pallet::FreezeReason
     **/
    PalletAssetRewardsFreezeReason: {
        _enum: string[];
    };
    /**
     * Lookup617: pallet_balances::pallet::Error<T, I>
     **/
    PalletBalancesError: {
        _enum: string[];
    };
    /**
     * Lookup618: pallet_transaction_payment::Releases
     **/
    PalletTransactionPaymentReleases: {
        _enum: string[];
    };
    /**
     * Lookup619: pallet_election_provider_multi_phase::ReadySolution<AccountId, MaxWinners>
     **/
    PalletElectionProviderMultiPhaseReadySolution: {
        supports: string;
        score: string;
        compute: string;
    };
    /**
     * Lookup621: pallet_election_provider_multi_phase::RoundSnapshot<sp_core::crypto::AccountId32, VoterType>
     **/
    PalletElectionProviderMultiPhaseRoundSnapshot: {
        voters: string;
        targets: string;
    };
    /**
     * Lookup628: pallet_election_provider_multi_phase::signed::SignedSubmission<sp_core::crypto::AccountId32, Balance, kitchensink_runtime::NposSolution16>
     **/
    PalletElectionProviderMultiPhaseSignedSignedSubmission: {
        who: string;
        deposit: string;
        rawSolution: string;
        callFee: string;
    };
    /**
     * Lookup629: pallet_election_provider_multi_phase::pallet::Error<T>
     **/
    PalletElectionProviderMultiPhaseError: {
        _enum: string[];
    };
    /**
     * Lookup630: pallet_staking::StakingLedger<T>
     **/
    PalletStakingStakingLedger: {
        stash: string;
        total: string;
        active: string;
        unlocking: string;
        legacyClaimedRewards: string;
    };
    /**
     * Lookup632: pallet_staking::Nominations<T>
     **/
    PalletStakingNominations: {
        targets: string;
        submittedIn: string;
        suppressed: string;
    };
    /**
     * Lookup633: pallet_staking::ActiveEraInfo
     **/
    PalletStakingActiveEraInfo: {
        index: string;
        start: string;
    };
    /**
     * Lookup636: sp_staking::Exposure<sp_core::crypto::AccountId32, Balance>
     **/
    SpStakingExposure: {
        total: string;
        own: string;
        others: string;
    };
    /**
     * Lookup638: sp_staking::IndividualExposure<sp_core::crypto::AccountId32, Balance>
     **/
    SpStakingIndividualExposure: {
        who: string;
        value: string;
    };
    /**
     * Lookup639: sp_staking::PagedExposureMetadata<Balance>
     **/
    SpStakingPagedExposureMetadata: {
        total: string;
        own: string;
        nominatorCount: string;
        pageCount: string;
    };
    /**
     * Lookup641: sp_staking::ExposurePage<sp_core::crypto::AccountId32, Balance>
     **/
    SpStakingExposurePage: {
        pageTotal: string;
        others: string;
    };
    /**
     * Lookup642: pallet_staking::EraRewardPoints<sp_core::crypto::AccountId32>
     **/
    PalletStakingEraRewardPoints: {
        total: string;
        individual: string;
    };
    /**
     * Lookup647: pallet_staking::UnappliedSlash<sp_core::crypto::AccountId32, Balance>
     **/
    PalletStakingUnappliedSlash: {
        validator: string;
        own: string;
        others: string;
        reporters: string;
        payout: string;
    };
    /**
     * Lookup649: pallet_staking::slashing::SlashingSpans
     **/
    PalletStakingSlashingSlashingSpans: {
        spanIndex: string;
        lastStart: string;
        lastNonzeroSlash: string;
        prior: string;
    };
    /**
     * Lookup650: pallet_staking::slashing::SpanRecord<Balance>
     **/
    PalletStakingSlashingSpanRecord: {
        slashed: string;
        paidOut: string;
    };
    /**
     * Lookup651: pallet_staking::pallet::pallet::Error<T>
     **/
    PalletStakingPalletError: {
        _enum: string[];
    };
    /**
     * Lookup658: sp_core::crypto::KeyTypeId
     **/
    SpCoreCryptoKeyTypeId: string;
    /**
     * Lookup659: pallet_session::pallet::Error<T>
     **/
    PalletSessionError: {
        _enum: string[];
    };
    /**
     * Lookup665: pallet_democracy::types::ReferendumInfo<BlockNumber, frame_support::traits::preimages::Bounded<kitchensink_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, Balance>
     **/
    PalletDemocracyReferendumInfo: {
        _enum: {
            Ongoing: string;
            Finished: {
                approved: string;
                end: string;
            };
        };
    };
    /**
     * Lookup666: pallet_democracy::types::ReferendumStatus<BlockNumber, frame_support::traits::preimages::Bounded<kitchensink_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, Balance>
     **/
    PalletDemocracyReferendumStatus: {
        end: string;
        proposal: string;
        threshold: string;
        delay: string;
        tally: string;
    };
    /**
     * Lookup667: pallet_democracy::types::Tally<Balance>
     **/
    PalletDemocracyTally: {
        ayes: string;
        nays: string;
        turnout: string;
    };
    /**
     * Lookup668: pallet_democracy::vote::Voting<Balance, sp_core::crypto::AccountId32, BlockNumber, MaxVotes>
     **/
    PalletDemocracyVoteVoting: {
        _enum: {
            Direct: {
                votes: string;
                delegations: string;
                prior: string;
            };
            Delegating: {
                balance: string;
                target: string;
                conviction: string;
                delegations: string;
                prior: string;
            };
        };
    };
    /**
     * Lookup672: pallet_democracy::types::Delegations<Balance>
     **/
    PalletDemocracyDelegations: {
        votes: string;
        capital: string;
    };
    /**
     * Lookup673: pallet_democracy::vote::PriorLock<BlockNumber, Balance>
     **/
    PalletDemocracyVotePriorLock: string;
    /**
     * Lookup676: pallet_democracy::pallet::Error<T>
     **/
    PalletDemocracyError: {
        _enum: string[];
    };
    /**
     * Lookup680: pallet_collective::Votes<sp_core::crypto::AccountId32, BlockNumber>
     **/
    PalletCollectiveVotes: {
        index: string;
        threshold: string;
        ayes: string;
        nays: string;
        end: string;
    };
    /**
     * Lookup681: pallet_collective::pallet::Error<T, I>
     **/
    PalletCollectiveError: {
        _enum: string[];
    };
    /**
     * Lookup685: pallet_elections_phragmen::SeatHolder<sp_core::crypto::AccountId32, Balance>
     **/
    PalletElectionsPhragmenSeatHolder: {
        who: string;
        stake: string;
        deposit: string;
    };
    /**
     * Lookup686: pallet_elections_phragmen::Voter<sp_core::crypto::AccountId32, Balance>
     **/
    PalletElectionsPhragmenVoter: {
        votes: string;
        stake: string;
        deposit: string;
    };
    /**
     * Lookup687: pallet_elections_phragmen::pallet::Error<T>
     **/
    PalletElectionsPhragmenError: {
        _enum: string[];
    };
    /**
     * Lookup689: pallet_membership::pallet::Error<T, I>
     **/
    PalletMembershipError: {
        _enum: string[];
    };
    /**
     * Lookup690: pallet_grandpa::StoredState<N>
     **/
    PalletGrandpaStoredState: {
        _enum: {
            Live: string;
            PendingPause: {
                scheduledAt: string;
                delay: string;
            };
            Paused: string;
            PendingResume: {
                scheduledAt: string;
                delay: string;
            };
        };
    };
    /**
     * Lookup691: pallet_grandpa::StoredPendingChange<N, Limit>
     **/
    PalletGrandpaStoredPendingChange: {
        scheduledAt: string;
        delay: string;
        nextAuthorities: string;
        forced: string;
    };
    /**
     * Lookup693: pallet_grandpa::pallet::Error<T>
     **/
    PalletGrandpaError: {
        _enum: string[];
    };
    /**
     * Lookup694: pallet_treasury::Proposal<sp_core::crypto::AccountId32, Balance>
     **/
    PalletTreasuryProposal: {
        proposer: string;
        value: string;
        beneficiary: string;
        bond: string;
    };
    /**
     * Lookup696: pallet_treasury::SpendStatus<frame_support::traits::tokens::fungible::union_of::NativeOrWithId<AssetId>, AssetBalance, sp_core::crypto::AccountId32, BlockNumber, PaymentId>
     **/
    PalletTreasurySpendStatus: {
        assetKind: string;
        amount: string;
        beneficiary: string;
        validFrom: string;
        expireAt: string;
        status: string;
    };
    /**
     * Lookup697: pallet_treasury::PaymentState<Id>
     **/
    PalletTreasuryPaymentState: {
        _enum: {
            Pending: string;
            Attempted: {
                id: string;
            };
            Failed: string;
        };
    };
    /**
     * Lookup698: frame_support::PalletId
     **/
    FrameSupportPalletId: string;
    /**
     * Lookup699: pallet_treasury::pallet::Error<T, I>
     **/
    PalletTreasuryError: {
        _enum: string[];
    };
    /**
     * Lookup700: pallet_asset_rate::pallet::Error<T>
     **/
    PalletAssetRateError: {
        _enum: string[];
    };
    /**
     * Lookup702: pallet_contracts::wasm::CodeInfo<T>
     **/
    PalletContractsWasmCodeInfo: {
        owner: string;
        deposit: string;
        refcount: string;
        determinism: string;
        codeLen: string;
    };
    /**
     * Lookup703: pallet_contracts::storage::ContractInfo<T>
     **/
    PalletContractsStorageContractInfo: {
        trieId: string;
        codeHash: string;
        storageBytes: string;
        storageItems: string;
        storageByteDeposit: string;
        storageItemDeposit: string;
        storageBaseDeposit: string;
        delegateDependencies: string;
    };
    /**
     * Lookup708: pallet_contracts::storage::DeletionQueueManager<T>
     **/
    PalletContractsStorageDeletionQueueManager: {
        insertCounter: string;
        deleteCounter: string;
    };
    /**
     * Lookup710: pallet_contracts::schedule::Schedule<T>
     **/
    PalletContractsSchedule: {
        limits: string;
        instructionWeights: string;
    };
    /**
     * Lookup711: pallet_contracts::schedule::Limits
     **/
    PalletContractsScheduleLimits: {
        eventTopics: string;
        memoryPages: string;
        subjectLen: string;
        payloadLen: string;
        runtimeMemory: string;
        validatorRuntimeMemory: string;
        eventRefTime: string;
    };
    /**
     * Lookup712: pallet_contracts::schedule::InstructionWeights<T>
     **/
    PalletContractsScheduleInstructionWeights: {
        base: string;
    };
    /**
     * Lookup713: pallet_contracts::Environment<T>
     **/
    PalletContractsEnvironment: {
        _alias: {
            hash_: string;
        };
        accountId: string;
        balance: string;
        hash_: string;
        hasher: string;
        timestamp: string;
        blockNumber: string;
    };
    /**
     * Lookup714: pallet_contracts::EnvironmentType<sp_core::crypto::AccountId32>
     **/
    PalletContractsEnvironmentTypeAccountId32: string;
    /**
     * Lookup715: pallet_contracts::EnvironmentType<T>
     **/
    PalletContractsEnvironmentTypeU128: string;
    /**
     * Lookup716: pallet_contracts::EnvironmentType<primitive_types::H256>
     **/
    PalletContractsEnvironmentTypeH256: string;
    /**
     * Lookup717: pallet_contracts::EnvironmentType<sp_runtime::traits::BlakeTwo256>
     **/
    PalletContractsEnvironmentTypeBlakeTwo256: string;
    /**
     * Lookup718: pallet_contracts::EnvironmentType<T>
     **/
    PalletContractsEnvironmentTypeU64: string;
    /**
     * Lookup719: pallet_contracts::EnvironmentType<T>
     **/
    PalletContractsEnvironmentTypeU32: string;
    /**
     * Lookup721: pallet_contracts::pallet::Error<T>
     **/
    PalletContractsError: {
        _enum: string[];
    };
    /**
     * Lookup722: pallet_sudo::pallet::Error<T>
     **/
    PalletSudoError: {
        _enum: string[];
    };
    /**
     * Lookup725: pallet_im_online::pallet::Error<T>
     **/
    PalletImOnlineError: {
        _enum: string[];
    };
    /**
     * Lookup728: sp_staking::offence::OffenceDetails<sp_core::crypto::AccountId32, Offender>
     **/
    SpStakingOffenceOffenceDetails: {
        offender: string;
        reporters: string;
    };
    /**
     * Lookup732: pallet_identity::types::Registration<Balance, MaxJudgements, pallet_identity::legacy::IdentityInfo<FieldLimit>>
     **/
    PalletIdentityRegistration: {
        judgements: string;
        deposit: string;
        info: string;
    };
    /**
     * Lookup740: pallet_identity::types::RegistrarInfo<Balance, sp_core::crypto::AccountId32, IdField>
     **/
    PalletIdentityRegistrarInfo: {
        account: string;
        fee: string;
        fields: string;
    };
    /**
     * Lookup743: pallet_identity::types::AuthorityProperties<sp_core::crypto::AccountId32>
     **/
    PalletIdentityAuthorityProperties: {
        accountId: string;
        allocation: string;
    };
    /**
     * Lookup744: pallet_identity::types::UsernameInformation<sp_core::crypto::AccountId32, Balance>
     **/
    PalletIdentityUsernameInformation: {
        owner: string;
        provider: string;
    };
    /**
     * Lookup745: pallet_identity::types::Provider<Balance>
     **/
    PalletIdentityProvider: {
        _enum: {
            Allocation: string;
            AuthorityDeposit: string;
            System: string;
        };
    };
    /**
     * Lookup747: pallet_identity::pallet::Error<T>
     **/
    PalletIdentityError: {
        _enum: string[];
    };
    /**
     * Lookup748: pallet_society::MemberRecord
     **/
    PalletSocietyMemberRecord: {
        rank: string;
        strikes: string;
        vouching: string;
        index: string;
    };
    /**
     * Lookup750: pallet_society::VouchingStatus
     **/
    PalletSocietyVouchingStatus: {
        _enum: string[];
    };
    /**
     * Lookup751: pallet_society::PayoutRecord<Balance, bounded_collections::bounded_vec::BoundedVec<T, S>>
     **/
    PalletSocietyPayoutRecord: {
        paid: string;
        payouts: string;
    };
    /**
     * Lookup756: pallet_society::Bid<sp_core::crypto::AccountId32, Balance>
     **/
    PalletSocietyBid: {
        who: string;
        kind: string;
        value: string;
    };
    /**
     * Lookup757: pallet_society::BidKind<sp_core::crypto::AccountId32, Balance>
     **/
    PalletSocietyBidKind: {
        _enum: {
            Deposit: string;
            Vouch: string;
        };
    };
    /**
     * Lookup759: pallet_society::Candidacy<sp_core::crypto::AccountId32, Balance>
     **/
    PalletSocietyCandidacy: {
        round: string;
        kind: string;
        bid: string;
        tally: string;
        skepticStruck: string;
    };
    /**
     * Lookup760: pallet_society::Tally
     **/
    PalletSocietyTally: {
        approvals: string;
        rejections: string;
    };
    /**
     * Lookup762: pallet_society::Vote
     **/
    PalletSocietyVote: {
        approve: string;
        weight: string;
    };
    /**
     * Lookup764: pallet_society::IntakeRecord<sp_core::crypto::AccountId32, Balance>
     **/
    PalletSocietyIntakeRecord: {
        who: string;
        bid: string;
        round: string;
    };
    /**
     * Lookup766: pallet_society::pallet::Error<T, I>
     **/
    PalletSocietyError: {
        _enum: string[];
    };
    /**
     * Lookup767: pallet_recovery::RecoveryConfig<BlockNumber, Balance, bounded_collections::bounded_vec::BoundedVec<sp_core::crypto::AccountId32, S>>
     **/
    PalletRecoveryRecoveryConfig: {
        delayPeriod: string;
        deposit: string;
        friends: string;
        threshold: string;
    };
    /**
     * Lookup769: pallet_recovery::ActiveRecovery<BlockNumber, Balance, bounded_collections::bounded_vec::BoundedVec<sp_core::crypto::AccountId32, S>>
     **/
    PalletRecoveryActiveRecovery: {
        created: string;
        deposit: string;
        friends: string;
    };
    /**
     * Lookup770: pallet_recovery::pallet::Error<T>
     **/
    PalletRecoveryError: {
        _enum: string[];
    };
    /**
     * Lookup773: pallet_vesting::Releases
     **/
    PalletVestingReleases: {
        _enum: string[];
    };
    /**
     * Lookup774: pallet_vesting::pallet::Error<T>
     **/
    PalletVestingError: {
        _enum: string[];
    };
    /**
     * Lookup777: pallet_scheduler::Scheduled<Name, frame_support::traits::preimages::Bounded<kitchensink_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, BlockNumber, kitchensink_runtime::OriginCaller, sp_core::crypto::AccountId32>
     **/
    PalletSchedulerScheduled: {
        maybeId: string;
        priority: string;
        call: string;
        maybePeriodic: string;
        origin: string;
    };
    /**
     * Lookup779: pallet_scheduler::RetryConfig<Period>
     **/
    PalletSchedulerRetryConfig: {
        totalRetries: string;
        remaining: string;
        period: string;
    };
    /**
     * Lookup780: pallet_scheduler::pallet::Error<T>
     **/
    PalletSchedulerError: {
        _enum: string[];
    };
    /**
     * Lookup781: pallet_glutton::pallet::Error<T>
     **/
    PalletGluttonError: {
        _enum: string[];
    };
    /**
     * Lookup782: pallet_preimage::OldRequestStatus<sp_core::crypto::AccountId32, Balance>
     **/
    PalletPreimageOldRequestStatus: {
        _enum: {
            Unrequested: {
                deposit: string;
                len: string;
            };
            Requested: {
                deposit: string;
                count: string;
                len: string;
            };
        };
    };
    /**
     * Lookup784: pallet_preimage::RequestStatus<sp_core::crypto::AccountId32, frame_support::traits::tokens::fungible::HoldConsideration<A, F, R, D, Fp>>
     **/
    PalletPreimageRequestStatus: {
        _enum: {
            Unrequested: {
                ticket: string;
                len: string;
            };
            Requested: {
                maybeTicket: string;
                count: string;
                maybeLen: string;
            };
        };
    };
    /**
     * Lookup789: pallet_preimage::pallet::Error<T>
     **/
    PalletPreimageError: {
        _enum: string[];
    };
    /**
     * Lookup792: pallet_proxy::ProxyDefinition<sp_core::crypto::AccountId32, kitchensink_runtime::ProxyType, BlockNumber>
     **/
    PalletProxyProxyDefinition: {
        delegate: string;
        proxyType: string;
        delay: string;
    };
    /**
     * Lookup796: pallet_proxy::Announcement<sp_core::crypto::AccountId32, primitive_types::H256, BlockNumber>
     **/
    PalletProxyAnnouncement: {
        real: string;
        callHash: string;
        height: string;
    };
    /**
     * Lookup798: pallet_proxy::pallet::Error<T>
     **/
    PalletProxyError: {
        _enum: string[];
    };
    /**
     * Lookup800: pallet_multisig::Multisig<BlockNumber, Balance, sp_core::crypto::AccountId32, MaxApprovals>
     **/
    PalletMultisigMultisig: {
        when: string;
        deposit: string;
        depositor: string;
        approvals: string;
    };
    /**
     * Lookup801: pallet_multisig::pallet::Error<T>
     **/
    PalletMultisigError: {
        _enum: string[];
    };
    /**
     * Lookup802: pallet_bounties::Bounty<sp_core::crypto::AccountId32, Balance, BlockNumber>
     **/
    PalletBountiesBounty: {
        proposer: string;
        value: string;
        fee: string;
        curatorDeposit: string;
        bond: string;
        status: string;
    };
    /**
     * Lookup803: pallet_bounties::BountyStatus<sp_core::crypto::AccountId32, BlockNumber>
     **/
    PalletBountiesBountyStatus: {
        _enum: {
            Proposed: string;
            Approved: string;
            Funded: string;
            CuratorProposed: {
                curator: string;
            };
            Active: {
                curator: string;
                updateDue: string;
            };
            PendingPayout: {
                curator: string;
                beneficiary: string;
                unlockAt: string;
            };
            ApprovedWithCurator: {
                curator: string;
            };
        };
    };
    /**
     * Lookup805: pallet_bounties::pallet::Error<T, I>
     **/
    PalletBountiesError: {
        _enum: string[];
    };
    /**
     * Lookup806: pallet_tips::OpenTip<sp_core::crypto::AccountId32, Balance, BlockNumber, primitive_types::H256>
     **/
    PalletTipsOpenTip: {
        reason: string;
        who: string;
        finder: string;
        deposit: string;
        closes: string;
        tips: string;
        findersFee: string;
    };
    /**
     * Lookup807: pallet_tips::pallet::Error<T, I>
     **/
    PalletTipsError: {
        _enum: string[];
    };
    /**
     * Lookup808: pallet_assets::types::AssetDetails<Balance, sp_core::crypto::AccountId32, DepositBalance>
     **/
    PalletAssetsAssetDetails: {
        owner: string;
        issuer: string;
        admin: string;
        freezer: string;
        supply: string;
        deposit: string;
        minBalance: string;
        isSufficient: string;
        accounts: string;
        sufficients: string;
        approvals: string;
        status: string;
    };
    /**
     * Lookup809: pallet_assets::types::AssetStatus
     **/
    PalletAssetsAssetStatus: {
        _enum: string[];
    };
    /**
     * Lookup810: pallet_assets::types::AssetAccount<Balance, DepositBalance, Extra, sp_core::crypto::AccountId32>
     **/
    PalletAssetsAssetAccount: {
        balance: string;
        status: string;
        reason: string;
        extra: string;
    };
    /**
     * Lookup811: pallet_assets::types::AccountStatus
     **/
    PalletAssetsAccountStatus: {
        _enum: string[];
    };
    /**
     * Lookup812: pallet_assets::types::ExistenceReason<Balance, sp_core::crypto::AccountId32>
     **/
    PalletAssetsExistenceReason: {
        _enum: {
            Consumer: string;
            Sufficient: string;
            DepositHeld: string;
            DepositRefunded: string;
            DepositFrom: string;
        };
    };
    /**
     * Lookup814: pallet_assets::types::Approval<Balance, DepositBalance>
     **/
    PalletAssetsApproval: {
        amount: string;
        deposit: string;
    };
    /**
     * Lookup815: pallet_assets::types::AssetMetadata<DepositBalance, bounded_collections::bounded_vec::BoundedVec<T, S>>
     **/
    PalletAssetsAssetMetadata: {
        deposit: string;
        name: string;
        symbol: string;
        decimals: string;
        isFrozen: string;
    };
    /**
     * Lookup817: pallet_assets::pallet::Error<T, I>
     **/
    PalletAssetsError: {
        _enum: string[];
    };
    /**
     * Lookup821: pallet_beefy::pallet::Error<T>
     **/
    PalletBeefyError: {
        _enum: string[];
    };
    /**
     * Lookup822: sp_consensus_beefy::mmr::BeefyAuthoritySet<primitive_types::H256>
     **/
    SpConsensusBeefyMmrBeefyAuthoritySet: {
        id: string;
        len: string;
        keysetCommitment: string;
    };
    /**
     * Lookup823: pallet_lottery::LotteryConfig<BlockNumber, Balance>
     **/
    PalletLotteryLotteryConfig: {
        price: string;
        start: string;
        length: string;
        delay: string;
        repeat: string;
    };
    /**
     * Lookup827: pallet_lottery::pallet::Error<T>
     **/
    PalletLotteryError: {
        _enum: string[];
    };
    /**
     * Lookup830: pallet_nis::pallet::Bid<Balance, sp_core::crypto::AccountId32>
     **/
    PalletNisBid: {
        amount: string;
        who: string;
    };
    /**
     * Lookup832: pallet_nis::pallet::SummaryRecord<BlockNumber, Balance>
     **/
    PalletNisSummaryRecord: {
        proportionOwed: string;
        index: string;
        thawed: string;
        lastPeriod: string;
        receiptsOnHold: string;
    };
    /**
     * Lookup833: pallet_nis::pallet::ReceiptRecord<sp_core::crypto::AccountId32, BlockNumber, Balance>
     **/
    PalletNisReceiptRecord: {
        proportion: string;
        owner: string;
        expiry: string;
    };
    /**
     * Lookup835: pallet_nis::pallet::Error<T>
     **/
    PalletNisError: {
        _enum: string[];
    };
    /**
     * Lookup836: pallet_uniques::types::CollectionDetails<sp_core::crypto::AccountId32, DepositBalance>
     **/
    PalletUniquesCollectionDetails: {
        owner: string;
        issuer: string;
        admin: string;
        freezer: string;
        totalDeposit: string;
        freeHolding: string;
        items: string;
        itemMetadatas: string;
        attributes: string;
        isFrozen: string;
    };
    /**
     * Lookup838: pallet_uniques::types::ItemDetails<sp_core::crypto::AccountId32, DepositBalance>
     **/
    PalletUniquesItemDetails: {
        owner: string;
        approved: string;
        isFrozen: string;
        deposit: string;
    };
    /**
     * Lookup839: pallet_uniques::types::CollectionMetadata<DepositBalance, StringLimit>
     **/
    PalletUniquesCollectionMetadata: {
        deposit: string;
        data: string;
        isFrozen: string;
    };
    /**
     * Lookup840: pallet_uniques::types::ItemMetadata<DepositBalance, StringLimit>
     **/
    PalletUniquesItemMetadata: {
        deposit: string;
        data: string;
        isFrozen: string;
    };
    /**
     * Lookup844: pallet_uniques::pallet::Error<T, I>
     **/
    PalletUniquesError: {
        _enum: string[];
    };
    /**
     * Lookup845: pallet_nfts::types::CollectionDetails<sp_core::crypto::AccountId32, DepositBalance>
     **/
    PalletNftsCollectionDetails: {
        owner: string;
        ownerDeposit: string;
        items: string;
        itemMetadatas: string;
        itemConfigs: string;
        attributes: string;
    };
    /**
     * Lookup847: pallet_nfts::types::CollectionRole
     **/
    PalletNftsCollectionRole: {
        _enum: string[];
    };
    /**
     * Lookup848: pallet_nfts::types::ItemDetails<sp_core::crypto::AccountId32, pallet_nfts::types::ItemDeposit<DepositBalance, sp_core::crypto::AccountId32>, bounded_collections::bounded_btree_map::BoundedBTreeMap<sp_core::crypto::AccountId32, Option<T>, S>>
     **/
    PalletNftsItemDetails: {
        owner: string;
        approvals: string;
        deposit: string;
    };
    /**
     * Lookup849: pallet_nfts::types::ItemDeposit<DepositBalance, sp_core::crypto::AccountId32>
     **/
    PalletNftsItemDeposit: {
        account: string;
        amount: string;
    };
    /**
     * Lookup854: pallet_nfts::types::CollectionMetadata<Deposit, StringLimit>
     **/
    PalletNftsCollectionMetadata: {
        deposit: string;
        data: string;
    };
    /**
     * Lookup855: pallet_nfts::types::ItemMetadata<pallet_nfts::types::ItemMetadataDeposit<DepositBalance, sp_core::crypto::AccountId32>, StringLimit>
     **/
    PalletNftsItemMetadata: {
        deposit: string;
        data: string;
    };
    /**
     * Lookup856: pallet_nfts::types::ItemMetadataDeposit<DepositBalance, sp_core::crypto::AccountId32>
     **/
    PalletNftsItemMetadataDeposit: {
        account: string;
        amount: string;
    };
    /**
     * Lookup859: pallet_nfts::types::AttributeDeposit<DepositBalance, sp_core::crypto::AccountId32>
     **/
    PalletNftsAttributeDeposit: {
        account: string;
        amount: string;
    };
    /**
     * Lookup862: pallet_nfts::types::PendingSwap<CollectionId, ItemId, pallet_nfts::types::PriceWithDirection<Amount>, Deadline>
     **/
    PalletNftsPendingSwap: {
        desiredCollection: string;
        desiredItem: string;
        price: string;
        deadline: string;
    };
    /**
     * Lookup864: pallet_nfts::types::PalletFeature
     **/
    PalletNftsPalletFeature: {
        _enum: string[];
    };
    /**
     * Lookup865: pallet_nfts::pallet::Error<T, I>
     **/
    PalletNftsError: {
        _enum: string[];
    };
    /**
     * Lookup866: pallet_nft_fractionalization::types::Details<AssetId, Fractions, Deposit, sp_core::crypto::AccountId32>
     **/
    PalletNftFractionalizationDetails: {
        asset: string;
        fractions: string;
        deposit: string;
        assetCreator: string;
    };
    /**
     * Lookup867: pallet_nft_fractionalization::pallet::Error<T>
     **/
    PalletNftFractionalizationError: {
        _enum: string[];
    };
    /**
     * Lookup868: pallet_salary::StatusType<CycleIndex, BlockNumber, Balance>
     **/
    PalletSalaryStatusType: {
        cycleIndex: string;
        cycleStart: string;
        budget: string;
        totalRegistrations: string;
        totalUnregisteredPaid: string;
    };
    /**
     * Lookup869: pallet_salary::ClaimantStatus<CycleIndex, Balance, Id>
     **/
    PalletSalaryClaimantStatus: {
        lastActive: string;
        status: string;
    };
    /**
     * Lookup870: pallet_salary::ClaimState<Balance, Id>
     **/
    PalletSalaryClaimState: {
        _enum: {
            Nothing: string;
            Registered: string;
            Attempted: {
                registered: string;
                id: string;
                amount: string;
            };
        };
    };
    /**
     * Lookup871: pallet_salary::pallet::Error<T, I>
     **/
    PalletSalaryError: {
        _enum: string[];
    };
    /**
     * Lookup872: pallet_core_fellowship::MemberStatus<BlockNumber>
     **/
    PalletCoreFellowshipMemberStatus: {
        isActive: string;
        lastPromotion: string;
        lastProof: string;
    };
    /**
     * Lookup874: pallet_core_fellowship::pallet::Error<T, I>
     **/
    PalletCoreFellowshipError: {
        _enum: string[];
    };
    /**
     * Lookup876: pallet_transaction_storage::TransactionInfo
     **/
    PalletTransactionStorageTransactionInfo: {
        _alias: {
            size_: string;
        };
        chunkRoot: string;
        contentHash: string;
        size_: string;
        blockChunks: string;
    };
    /**
     * Lookup878: pallet_transaction_storage::pallet::Error<T>
     **/
    PalletTransactionStorageError: {
        _enum: string[];
    };
    /**
     * Lookup879: pallet_bags_list::list::Node<T, I>
     **/
    PalletBagsListListNode: {
        id: string;
        prev: string;
        next: string;
        bagUpper: string;
        score: string;
    };
    /**
     * Lookup880: pallet_bags_list::list::Bag<T, I>
     **/
    PalletBagsListListBag: {
        head: string;
        tail: string;
    };
    /**
     * Lookup882: pallet_bags_list::pallet::Error<T, I>
     **/
    PalletBagsListError: {
        _enum: {
            List: string;
        };
    };
    /**
     * Lookup883: pallet_bags_list::list::ListError
     **/
    PalletBagsListListListError: {
        _enum: string[];
    };
    /**
     * Lookup884: pallet_child_bounties::ChildBounty<sp_core::crypto::AccountId32, Balance, BlockNumber>
     **/
    PalletChildBountiesChildBounty: {
        parentBounty: string;
        value: string;
        fee: string;
        curatorDeposit: string;
        status: string;
    };
    /**
     * Lookup885: pallet_child_bounties::ChildBountyStatus<sp_core::crypto::AccountId32, BlockNumber>
     **/
    PalletChildBountiesChildBountyStatus: {
        _enum: {
            Added: string;
            CuratorProposed: {
                curator: string;
            };
            Active: {
                curator: string;
            };
            PendingPayout: {
                curator: string;
                beneficiary: string;
                unlockAt: string;
            };
        };
    };
    /**
     * Lookup886: pallet_child_bounties::pallet::Error<T>
     **/
    PalletChildBountiesError: {
        _enum: string[];
    };
    /**
     * Lookup887: pallet_referenda::types::ReferendumInfo<TrackId, kitchensink_runtime::OriginCaller, Moment, frame_support::traits::preimages::Bounded<kitchensink_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, Balance, pallet_conviction_voting::types::Tally<Votes, Total>, sp_core::crypto::AccountId32, ScheduleAddress>
     **/
    PalletReferendaReferendumInfoConvictionVotingTally: {
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
     * Lookup888: pallet_referenda::types::ReferendumStatus<TrackId, kitchensink_runtime::OriginCaller, Moment, frame_support::traits::preimages::Bounded<kitchensink_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, Balance, pallet_conviction_voting::types::Tally<Votes, Total>, sp_core::crypto::AccountId32, ScheduleAddress>
     **/
    PalletReferendaReferendumStatusConvictionVotingTally: {
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
     * Lookup889: pallet_referenda::types::Deposit<sp_core::crypto::AccountId32, Balance>
     **/
    PalletReferendaDeposit: {
        who: string;
        amount: string;
    };
    /**
     * Lookup892: pallet_referenda::types::DecidingStatus<BlockNumber>
     **/
    PalletReferendaDecidingStatus: {
        since: string;
        confirming: string;
    };
    /**
     * Lookup898: pallet_referenda::types::TrackDetails<Balance, Moment, Name>
     **/
    PalletReferendaTrackDetails: {
        name: string;
        maxDeciding: string;
        decisionDeposit: string;
        preparePeriod: string;
        decisionPeriod: string;
        confirmPeriod: string;
        minEnactmentPeriod: string;
        minApproval: string;
        minSupport: string;
    };
    /**
     * Lookup899: pallet_referenda::pallet::Error<T, I>
     **/
    PalletReferendaError: {
        _enum: string[];
    };
    /**
     * Lookup900: pallet_remark::pallet::Error<T>
     **/
    PalletRemarkError: {
        _enum: string[];
    };
    /**
     * Lookup902: pallet_conviction_voting::vote::Voting<Balance, sp_core::crypto::AccountId32, BlockNumber, PollIndex, MaxVotes>
     **/
    PalletConvictionVotingVoteVoting: {
        _enum: {
            Casting: string;
            Delegating: string;
        };
    };
    /**
     * Lookup903: pallet_conviction_voting::vote::Casting<Balance, BlockNumber, PollIndex, MaxVotes>
     **/
    PalletConvictionVotingVoteCasting: {
        votes: string;
        delegations: string;
        prior: string;
    };
    /**
     * Lookup907: pallet_conviction_voting::types::Delegations<Balance>
     **/
    PalletConvictionVotingDelegations: {
        votes: string;
        capital: string;
    };
    /**
     * Lookup908: pallet_conviction_voting::vote::PriorLock<BlockNumber, Balance>
     **/
    PalletConvictionVotingVotePriorLock: string;
    /**
     * Lookup909: pallet_conviction_voting::vote::Delegating<Balance, sp_core::crypto::AccountId32, BlockNumber>
     **/
    PalletConvictionVotingVoteDelegating: {
        balance: string;
        target: string;
        conviction: string;
        delegations: string;
        prior: string;
    };
    /**
     * Lookup913: pallet_conviction_voting::pallet::Error<T, I>
     **/
    PalletConvictionVotingError: {
        _enum: string[];
    };
    /**
     * Lookup914: pallet_whitelist::pallet::Error<T>
     **/
    PalletWhitelistError: {
        _enum: string[];
    };
    /**
     * Lookup919: pallet_alliance::MemberRole
     **/
    PalletAllianceMemberRole: {
        _enum: string[];
    };
    /**
     * Lookup923: pallet_alliance::pallet::Error<T, I>
     **/
    PalletAllianceError: {
        _enum: string[];
    };
    /**
     * Lookup924: pallet_nomination_pools::PoolMember<T>
     **/
    PalletNominationPoolsPoolMember: {
        poolId: string;
        points: string;
        lastRecordedRewardCounter: string;
        unbondingEras: string;
    };
    /**
     * Lookup927: pallet_nomination_pools::BondedPoolInner<T>
     **/
    PalletNominationPoolsBondedPoolInner: {
        commission: string;
        memberCounter: string;
        points: string;
        roles: string;
        state: string;
    };
    /**
     * Lookup928: pallet_nomination_pools::Commission<T>
     **/
    PalletNominationPoolsCommission: {
        current: string;
        max: string;
        changeRate: string;
        throttleFrom: string;
        claimPermission: string;
    };
    /**
     * Lookup930: pallet_nomination_pools::PoolRoles<sp_core::crypto::AccountId32>
     **/
    PalletNominationPoolsPoolRoles: {
        depositor: string;
        root: string;
        nominator: string;
        bouncer: string;
    };
    /**
     * Lookup931: pallet_nomination_pools::RewardPool<T>
     **/
    PalletNominationPoolsRewardPool: {
        lastRecordedRewardCounter: string;
        lastRecordedTotalPayouts: string;
        totalRewardsClaimed: string;
        totalCommissionPending: string;
        totalCommissionClaimed: string;
    };
    /**
     * Lookup932: pallet_nomination_pools::SubPools<T>
     **/
    PalletNominationPoolsSubPools: {
        noEra: string;
        withEra: string;
    };
    /**
     * Lookup933: pallet_nomination_pools::UnbondPool<T>
     **/
    PalletNominationPoolsUnbondPool: {
        points: string;
        balance: string;
    };
    /**
     * Lookup938: pallet_nomination_pools::pallet::Error<T>
     **/
    PalletNominationPoolsError: {
        _enum: {
            PoolNotFound: string;
            PoolMemberNotFound: string;
            RewardPoolNotFound: string;
            SubPoolsNotFound: string;
            AccountBelongsToOtherPool: string;
            FullyUnbonding: string;
            MaxUnbondingLimit: string;
            CannotWithdrawAny: string;
            MinimumBondNotMet: string;
            OverflowRisk: string;
            NotDestroying: string;
            NotNominator: string;
            NotKickerOrDestroying: string;
            NotOpen: string;
            MaxPools: string;
            MaxPoolMembers: string;
            CanNotChangeState: string;
            DoesNotHavePermission: string;
            MetadataExceedsMaxLen: string;
            Defensive: string;
            PartialUnbondNotAllowedPermissionlessly: string;
            MaxCommissionRestricted: string;
            CommissionExceedsMaximum: string;
            CommissionExceedsGlobalMaximum: string;
            CommissionChangeThrottled: string;
            CommissionChangeRateNotAllowed: string;
            NoPendingCommission: string;
            NoCommissionCurrentSet: string;
            PoolIdInUse: string;
            InvalidPoolId: string;
            BondExtraRestricted: string;
            NothingToAdjust: string;
            NothingToSlash: string;
            SlashTooLow: string;
            AlreadyMigrated: string;
            NotMigrated: string;
            NotSupported: string;
            Restricted: string;
        };
    };
    /**
     * Lookup939: pallet_nomination_pools::pallet::DefensiveError
     **/
    PalletNominationPoolsDefensiveError: {
        _enum: string[];
    };
    /**
     * Lookup940: pallet_referenda::types::ReferendumInfo<TrackId, kitchensink_runtime::OriginCaller, Moment, frame_support::traits::preimages::Bounded<kitchensink_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, Balance, pallet_ranked_collective::Tally<T, I, M>, sp_core::crypto::AccountId32, ScheduleAddress>
     **/
    PalletReferendaReferendumInfoRankedCollectiveTally: {
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
     * Lookup941: pallet_referenda::types::ReferendumStatus<TrackId, kitchensink_runtime::OriginCaller, Moment, frame_support::traits::preimages::Bounded<kitchensink_runtime::RuntimeCall, sp_runtime::traits::BlakeTwo256>, Balance, pallet_ranked_collective::Tally<T, I, M>, sp_core::crypto::AccountId32, ScheduleAddress>
     **/
    PalletReferendaReferendumStatusRankedCollectiveTally: {
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
     * Lookup944: pallet_ranked_collective::MemberRecord
     **/
    PalletRankedCollectiveMemberRecord: {
        rank: string;
    };
    /**
     * Lookup948: pallet_ranked_collective::pallet::Error<T, I>
     **/
    PalletRankedCollectiveError: {
        _enum: string[];
    };
    /**
     * Lookup949: pallet_asset_conversion::types::PoolInfo<PoolAssetId>
     **/
    PalletAssetConversionPoolInfo: {
        lpToken: string;
    };
    /**
     * Lookup950: pallet_asset_conversion::pallet::Error<T>
     **/
    PalletAssetConversionError: {
        _enum: string[];
    };
    /**
     * Lookup951: pallet_fast_unstake::types::UnstakeRequest<T>
     **/
    PalletFastUnstakeUnstakeRequest: {
        stashes: string;
        checked: string;
    };
    /**
     * Lookup954: pallet_fast_unstake::pallet::Error<T>
     **/
    PalletFastUnstakeError: {
        _enum: string[];
    };
    /**
     * Lookup955: pallet_message_queue::BookState<MessageOrigin>
     **/
    PalletMessageQueueBookState: {
        _alias: {
            size_: string;
        };
        begin: string;
        end: string;
        count: string;
        readyNeighbours: string;
        messageCount: string;
        size_: string;
    };
    /**
     * Lookup957: pallet_message_queue::Neighbours<MessageOrigin>
     **/
    PalletMessageQueueNeighbours: {
        prev: string;
        next: string;
    };
    /**
     * Lookup958: pallet_message_queue::Page<Size, HeapSize>
     **/
    PalletMessageQueuePage: {
        remaining: string;
        remainingSize: string;
        firstIndex: string;
        first: string;
        last: string;
        heap: string;
    };
    /**
     * Lookup960: pallet_message_queue::pallet::Error<T>
     **/
    PalletMessageQueueError: {
        _enum: string[];
    };
    /**
     * Lookup961: pallet_tx_pause::pallet::Error<T>
     **/
    PalletTxPauseError: {
        _enum: string[];
    };
    /**
     * Lookup962: pallet_safe_mode::pallet::Error<T>
     **/
    PalletSafeModeError: {
        _enum: string[];
    };
    /**
     * Lookup963: pallet_migrations::pallet::Error<T>
     **/
    PalletMigrationsError: {
        _enum: string[];
    };
    /**
     * Lookup967: pallet_broker::types::LeaseRecordItem
     **/
    PalletBrokerLeaseRecordItem: {
        until: string;
        task: string;
    };
    /**
     * Lookup969: pallet_broker::types::StatusRecord
     **/
    PalletBrokerStatusRecord: {
        coreCount: string;
        privatePoolSize: string;
        systemPoolSize: string;
        lastCommittedTimeslice: string;
        lastTimeslice: string;
    };
    /**
     * Lookup970: pallet_broker::types::SaleInfoRecord<Balance, RelayBlockNumber>
     **/
    PalletBrokerSaleInfoRecord: {
        saleStart: string;
        leadinLength: string;
        endPrice: string;
        regionBegin: string;
        regionEnd: string;
        idealCoresSold: string;
        coresOffered: string;
        firstCore: string;
        selloutPrice: string;
        coresSold: string;
    };
    /**
     * Lookup971: pallet_broker::types::PotentialRenewalId
     **/
    PalletBrokerPotentialRenewalId: {
        core: string;
        when: string;
    };
    /**
     * Lookup972: pallet_broker::types::PotentialRenewalRecord<Balance>
     **/
    PalletBrokerPotentialRenewalRecord: {
        price: string;
        completion: string;
    };
    /**
     * Lookup973: pallet_broker::types::CompletionStatus
     **/
    PalletBrokerCompletionStatus: {
        _enum: {
            Partial: string;
            Complete: string;
        };
    };
    /**
     * Lookup974: pallet_broker::types::RegionRecord<sp_core::crypto::AccountId32, Balance>
     **/
    PalletBrokerRegionRecord: {
        end: string;
        owner: string;
        paid: string;
    };
    /**
     * Lookup976: pallet_broker::types::ContributionRecord<sp_core::crypto::AccountId32>
     **/
    PalletBrokerContributionRecord: {
        length: string;
        payee: string;
    };
    /**
     * Lookup977: pallet_broker::types::PoolIoRecord
     **/
    PalletBrokerPoolIoRecord: {
        private: string;
        system: string;
    };
    /**
     * Lookup979: pallet_broker::types::InstaPoolHistoryRecord<Balance>
     **/
    PalletBrokerInstaPoolHistoryRecord: {
        privateContributions: string;
        systemContributions: string;
        maybePayout: string;
    };
    /**
     * Lookup981: pallet_broker::types::AutoRenewalRecord
     **/
    PalletBrokerAutoRenewalRecord: {
        core: string;
        task: string;
        nextRenewal: string;
    };
    /**
     * Lookup983: pallet_broker::pallet::Error<T>
     **/
    PalletBrokerError: {
        _enum: string[];
    };
    /**
     * Lookup984: pallet_example_tasks::pallet::Error<T>
     **/
    PalletExampleTasksError: {
        _enum: string[];
    };
    /**
     * Lookup985: pallet_asset_conversion_ops::pallet::Error<T>
     **/
    PalletAssetConversionOpsError: {
        _enum: string[];
    };
    /**
     * Lookup987: pallet_revive::wasm::CodeInfo<T>
     **/
    PalletReviveWasmCodeInfo: {
        owner: string;
        deposit: string;
        refcount: string;
        codeLen: string;
        behaviourVersion: string;
    };
    /**
     * Lookup988: pallet_revive::storage::ContractInfo<T>
     **/
    PalletReviveStorageContractInfo: {
        trieId: string;
        codeHash: string;
        storageBytes: string;
        storageItems: string;
        storageByteDeposit: string;
        storageItemDeposit: string;
        storageBaseDeposit: string;
        immutableDataLen: string;
    };
    /**
     * Lookup990: pallet_revive::storage::DeletionQueueManager<T>
     **/
    PalletReviveStorageDeletionQueueManager: {
        insertCounter: string;
        deleteCounter: string;
    };
    /**
     * Lookup991: pallet_revive::pallet::Error<T>
     **/
    PalletReviveError: {
        _enum: string[];
    };
    /**
     * Lookup992: pallet_delegated_staking::types::Delegation<T>
     **/
    PalletDelegatedStakingDelegation: {
        agent: string;
        amount: string;
    };
    /**
     * Lookup993: pallet_delegated_staking::types::AgentLedger<T>
     **/
    PalletDelegatedStakingAgentLedger: {
        payee: string;
        totalDelegated: string;
        unclaimedWithdrawals: string;
        pendingSlash: string;
    };
    /**
     * Lookup994: pallet_delegated_staking::pallet::Error<T>
     **/
    PalletDelegatedStakingError: {
        _enum: string[];
    };
    /**
     * Lookup995: pallet_asset_rewards::PoolStakerInfo<Balance>
     **/
    PalletAssetRewardsPoolStakerInfo: {
        amount: string;
        rewards: string;
        rewardPerTokenPaid: string;
    };
    /**
     * Lookup996: pallet_asset_rewards::PoolInfo<sp_core::crypto::AccountId32, frame_support::traits::tokens::fungible::union_of::NativeOrWithId<AssetId>, Balance, BlockNumber>
     **/
    PalletAssetRewardsPoolInfo: {
        stakedAssetId: string;
        rewardAssetId: string;
        rewardRatePerBlock: string;
        expiryBlock: string;
        admin: string;
        totalTokensStaked: string;
        rewardPerTokenStored: string;
        lastUpdateBlock: string;
        account: string;
    };
    /**
     * Lookup999: pallet_asset_rewards::pallet::Error<T>
     **/
    PalletAssetRewardsError: {
        _enum: string[];
    };
    /**
     * Lookup1000: pallet_assets_freezer::pallet::Error<T, I>
     **/
    PalletAssetsFreezerError: {
        _enum: string[];
    };
    /**
     * Lookup1001: pallet_meta_tx::pallet::Error<T>
     **/
    PalletMetaTxError: {
        _enum: string[];
    };
    /**
     * Lookup1003: frame_system::extensions::check_weight::CheckWeight<T>
     **/
    FrameSystemExtensionsCheckWeight: string;
    /**
     * Lookup1004: pallet_asset_conversion_tx_payment::ChargeAssetTxPayment<T>
     **/
    PalletAssetConversionTxPaymentChargeAssetTxPayment: {
        tip: string;
        assetId: string;
    };
    /**
     * Lookup1006: frame_system::extensions::weight_reclaim::WeightReclaim<T>
     **/
    FrameSystemExtensionsWeightReclaim: string;
    /**
     * Lookup1007: sp_runtime::generic::block::Block<sp_runtime::generic::header::Header<Number, Hash>, sp_runtime::generic::unchecked_extrinsic::UncheckedExtrinsic<sp_runtime::multiaddress::MultiAddress<sp_core::crypto::AccountId32, AccountIndex>, kitchensink_runtime::RuntimeCall, sp_runtime::MultiSignature, Extra>>
     **/
    SpRuntimeBlock: {
        header: string;
        extrinsics: string;
    };
    /**
     * Lookup1010: sp_runtime::ExtrinsicInclusionMode
     **/
    SpRuntimeExtrinsicInclusionMode: {
        _enum: string[];
    };
    /**
     * Lookup1013: frame_support::view_functions::ViewFunctionId
     **/
    FrameSupportViewFunctionsViewFunctionId: {
        prefix: string;
        suffix: string;
    };
    /**
     * Lookup1015: frame_support::view_functions::ViewFunctionDispatchError
     **/
    FrameSupportViewFunctionsViewFunctionDispatchError: {
        _enum: {
            NotImplemented: string;
            NotFound: string;
            Codec: string;
        };
    };
    /**
     * Lookup1017: sp_runtime::transaction_validity::TransactionValidityError
     **/
    SpRuntimeTransactionValidityTransactionValidityError: {
        _enum: {
            Invalid: string;
            Unknown: string;
        };
    };
    /**
     * Lookup1018: sp_runtime::transaction_validity::InvalidTransaction
     **/
    SpRuntimeTransactionValidityInvalidTransaction: {
        _enum: {
            Call: string;
            Payment: string;
            Future: string;
            Stale: string;
            BadProof: string;
            AncientBirthBlock: string;
            ExhaustsResources: string;
            Custom: string;
            BadMandatory: string;
            MandatoryValidation: string;
            BadSigner: string;
            IndeterminateImplicit: string;
            UnknownOrigin: string;
        };
    };
    /**
     * Lookup1019: sp_runtime::transaction_validity::UnknownTransaction
     **/
    SpRuntimeTransactionValidityUnknownTransaction: {
        _enum: {
            CannotLookup: string;
            NoUnsignedValidator: string;
            Custom: string;
        };
    };
    /**
     * Lookup1020: sp_inherents::InherentData
     **/
    SpInherentsInherentData: {
        data: string;
    };
    /**
     * Lookup1024: sp_inherents::CheckInherentsResult
     **/
    SpInherentsCheckInherentsResult: {
        okay: string;
        fatalError: string;
        errors: string;
    };
    /**
     * Lookup1025: sp_runtime::transaction_validity::TransactionSource
     **/
    SpRuntimeTransactionValidityTransactionSource: {
        _enum: string[];
    };
    /**
     * Lookup1027: sp_runtime::transaction_validity::ValidTransaction
     **/
    SpRuntimeTransactionValidityValidTransaction: {
        priority: string;
        requires: string;
        provides: string;
        longevity: string;
        propagate: string;
    };
    /**
     * Lookup1028: sp_statement_store::runtime_api::StatementSource
     **/
    SpStatementStoreRuntimeApiStatementSource: {
        _enum: string[];
    };
    /**
     * Lookup1030: sp_statement_store::runtime_api::ValidStatement
     **/
    SpStatementStoreRuntimeApiValidStatement: {
        maxCount: string;
        maxSize: string;
    };
    /**
     * Lookup1031: sp_statement_store::runtime_api::InvalidStatement
     **/
    SpStatementStoreRuntimeApiInvalidStatement: {
        _enum: string[];
    };
    /**
     * Lookup1035: sp_consensus_babe::BabeConfiguration
     **/
    SpConsensusBabeBabeConfiguration: {
        slotDuration: string;
        epochLength: string;
        c: string;
        authorities: string;
        randomness: string;
        allowedSlots: string;
    };
    /**
     * Lookup1036: sp_consensus_babe::Epoch
     **/
    SpConsensusBabeEpoch: {
        epochIndex: string;
        startSlot: string;
        duration: string;
        authorities: string;
        randomness: string;
        config: string;
    };
    /**
     * Lookup1041: pallet_contracts::primitives::ExecReturnValue
     **/
    PalletContractsPrimitivesExecReturnValue: {
        flags: string;
        data: string;
    };
    /**
     * Lookup1042: pallet_contracts_uapi::flags::ReturnFlags
     **/
    PalletContractsUapiFlagsReturnFlags: {
        bits: string;
    };
    /**
     * Lookup1043: pallet_contracts::primitives::StorageDeposit<Balance>
     **/
    PalletContractsPrimitivesStorageDeposit: {
        _enum: {
            Refund: string;
            Charge: string;
        };
    };
    /**
     * Lookup1046: pallet_contracts::primitives::Code<primitive_types::H256>
     **/
    PalletContractsPrimitivesCode: {
        _enum: {
            Upload: string;
            Existing: string;
        };
    };
    /**
     * Lookup1047: pallet_contracts::primitives::ContractResult<Result<pallet_contracts::primitives::InstantiateReturnValue<sp_core::crypto::AccountId32>, sp_runtime::DispatchError>, Balance, frame_system::EventRecord<kitchensink_runtime::RuntimeEvent, primitive_types::H256>>
     **/
    PalletContractsPrimitivesContractResult: {
        gasConsumed: string;
        gasRequired: string;
        storageDeposit: string;
        debugMessage: string;
        result: string;
        events: string;
    };
    /**
     * Lookup1049: pallet_contracts::primitives::InstantiateReturnValue<sp_core::crypto::AccountId32>
     **/
    PalletContractsPrimitivesInstantiateReturnValue: {
        result: string;
        accountId: string;
    };
    /**
     * Lookup1051: pallet_contracts::primitives::CodeUploadReturnValue<primitive_types::H256, Balance>
     **/
    PalletContractsPrimitivesCodeUploadReturnValue: {
        codeHash: string;
        deposit: string;
    };
    /**
     * Lookup1053: pallet_contracts::primitives::ContractAccessError
     **/
    PalletContractsPrimitivesContractAccessError: {
        _enum: string[];
    };
    /**
     * Lookup1056: pallet_revive::primitives::ContractResult<pallet_revive::primitives::ExecReturnValue, Balance>
     **/
    PalletRevivePrimitivesContractResultExecReturnValue: {
        gasConsumed: string;
        gasRequired: string;
        storageDeposit: string;
        result: string;
    };
    /**
     * Lookup1057: pallet_revive::primitives::ExecReturnValue
     **/
    PalletRevivePrimitivesExecReturnValue: {
        flags: string;
        data: string;
    };
    /**
     * Lookup1058: pallet_revive_uapi::flags::ReturnFlags
     **/
    PalletReviveUapiFlagsReturnFlags: {
        bits: string;
    };
    /**
     * Lookup1059: pallet_revive::primitives::StorageDeposit<Balance>
     **/
    PalletRevivePrimitivesStorageDeposit: {
        _enum: {
            Refund: string;
            Charge: string;
        };
    };
    /**
     * Lookup1061: pallet_revive::primitives::Code
     **/
    PalletRevivePrimitivesCode: {
        _enum: {
            Upload: string;
            Existing: string;
        };
    };
    /**
     * Lookup1062: pallet_revive::primitives::ContractResult<pallet_revive::primitives::InstantiateReturnValue, Balance>
     **/
    PalletRevivePrimitivesContractResultInstantiateReturnValue: {
        gasConsumed: string;
        gasRequired: string;
        storageDeposit: string;
        result: string;
    };
    /**
     * Lookup1063: pallet_revive::primitives::InstantiateReturnValue
     **/
    PalletRevivePrimitivesInstantiateReturnValue: {
        result: string;
        addr: string;
    };
    /**
     * Lookup1065: pallet_revive::evm::api::rpc_types_gen::GenericTransaction
     **/
    PalletReviveEvmApiRpcTypesGenGenericTransaction: {
        _alias: {
            r_type: string;
        };
        accessList: string;
        blobVersionedHashes: string;
        blobs: string;
        chainId: string;
        from: string;
        gas: string;
        gasPrice: string;
        input: string;
        maxFeePerBlobGas: string;
        maxFeePerGas: string;
        maxPriorityFeePerGas: string;
        nonce: string;
        to: string;
        r_type: string;
        value: string;
    };
    /**
     * Lookup1068: pallet_revive::evm::api::rpc_types_gen::AccessListEntry
     **/
    PalletReviveEvmApiRpcTypesGenAccessListEntry: {
        address: string;
        storageKeys: string;
    };
    /**
     * Lookup1073: pallet_revive::evm::api::rpc_types_gen::InputOrData
     **/
    PalletReviveEvmApiRpcTypesGenInputOrData: {
        input: string;
        data: string;
    };
    /**
     * Lookup1078: pallet_revive::primitives::EthTransactInfo<Balance>
     **/
    PalletRevivePrimitivesEthTransactInfo: {
        gasRequired: string;
        storageDeposit: string;
        ethGas: string;
        data: string;
    };
    /**
     * Lookup1079: pallet_revive::primitives::EthTransactError
     **/
    PalletRevivePrimitivesEthTransactError: {
        _enum: {
            Data: string;
            Message: string;
        };
    };
    /**
     * Lookup1081: pallet_revive::primitives::CodeUploadReturnValue<Balance>
     **/
    PalletRevivePrimitivesCodeUploadReturnValue: {
        codeHash: string;
        deposit: string;
    };
    /**
     * Lookup1083: pallet_revive::primitives::ContractAccessError
     **/
    PalletRevivePrimitivesContractAccessError: {
        _enum: string[];
    };
    /**
     * Lookup1084: pallet_revive::evm::api::debug_rpc_types::TracerType
     **/
    PalletReviveEvmApiDebugRpcTypesTracerType: {
        _enum: {
            CallTracer: string;
        };
    };
    /**
     * Lookup1086: pallet_revive::evm::api::debug_rpc_types::CallTracerConfig
     **/
    PalletReviveEvmApiDebugRpcTypesCallTracerConfig: {
        withLogs: string;
        onlyTopCall: string;
    };
    /**
     * Lookup1089: pallet_revive::evm::api::debug_rpc_types::Trace
     **/
    PalletReviveEvmApiDebugRpcTypesTrace: {
        _enum: {
            Call: string;
        };
    };
    /**
     * Lookup1090: pallet_revive::evm::api::debug_rpc_types::CallTrace<primitive_types::U256>
     **/
    PalletReviveEvmApiDebugRpcTypesCallTrace: {
        from: string;
        gas: string;
        gasUsed: string;
        to: string;
        input: string;
        output: string;
        error: string;
        revertReason: string;
        calls: string;
        logs: string;
        value: string;
        callType: string;
    };
    /**
     * Lookup1094: pallet_revive::evm::api::debug_rpc_types::CallLog
     **/
    PalletReviveEvmApiDebugRpcTypesCallLog: {
        address: string;
        topics: string;
        data: string;
        position: string;
    };
    /**
     * Lookup1095: pallet_revive::evm::api::debug_rpc_types::CallType
     **/
    PalletReviveEvmApiDebugRpcTypesCallType: {
        _enum: string[];
    };
    /**
     * Lookup1098: pallet_transaction_payment::types::RuntimeDispatchInfo<Balance, sp_weights::weight_v2::Weight>
     **/
    PalletTransactionPaymentRuntimeDispatchInfo: {
        weight: string;
        class: string;
        partialFee: string;
    };
    /**
     * Lookup1099: pallet_transaction_payment::types::FeeDetails<Balance>
     **/
    PalletTransactionPaymentFeeDetails: {
        inclusionFee: string;
        tip: string;
    };
    /**
     * Lookup1101: pallet_transaction_payment::types::InclusionFee<Balance>
     **/
    PalletTransactionPaymentInclusionFee: {
        baseFee: string;
        lenFee: string;
        adjustedWeightFee: string;
    };
    /**
     * Lookup1105: sp_consensus_beefy::ValidatorSet<sp_consensus_beefy::ecdsa_crypto::Public>
     **/
    SpConsensusBeefyValidatorSet: {
        validators: string;
        id: string;
    };
    /**
     * Lookup1106: sp_consensus_beefy::ForkVotingProof<sp_runtime::generic::header::Header<Number, Hash>, sp_consensus_beefy::ecdsa_crypto::Public, sp_runtime::OpaqueValue>
     **/
    SpConsensusBeefyForkVotingProofOpaqueValue: {
        vote: string;
        ancestryProof: string;
        header: string;
    };
    /**
     * Lookup1108: sp_mmr_primitives::Error
     **/
    SpMmrPrimitivesError: {
        _enum: string[];
    };
    /**
     * Lookup1114: sp_mmr_primitives::LeafProof<primitive_types::H256>
     **/
    SpMmrPrimitivesLeafProof: {
        leafIndices: string;
        leafCount: string;
        items: string;
    };
    /**
     * Lookup1116: sp_mixnet::types::SessionStatus
     **/
    SpMixnetSessionStatus: {
        currentIndex: string;
        phase: string;
    };
    /**
     * Lookup1117: sp_mixnet::types::SessionPhase
     **/
    SpMixnetSessionPhase: {
        _enum: string[];
    };
    /**
     * Lookup1120: sp_mixnet::types::Mixnode
     **/
    SpMixnetMixnode: {
        kxPublic: string;
        peerId: string;
        externalAddresses: string;
    };
    /**
     * Lookup1121: sp_mixnet::types::MixnodesErr
     **/
    SpMixnetMixnodesErr: {
        _enum: {
            InsufficientRegistrations: {
                num: string;
                min: string;
            };
        };
    };
    /**
     * Lookup1127: kitchensink_runtime::RuntimeError
     **/
    KitchensinkRuntimeRuntimeError: {
        _enum: {
            System: string;
            Utility: string;
            Babe: string;
            __Unused3: string;
            __Unused4: string;
            Indices: string;
            Balances: string;
            __Unused7: string;
            __Unused8: string;
            __Unused9: string;
            ElectionProviderMultiPhase: string;
            Staking: string;
            Session: string;
            Democracy: string;
            Council: string;
            TechnicalCommittee: string;
            Elections: string;
            TechnicalMembership: string;
            Grandpa: string;
            Treasury: string;
            AssetRate: string;
            Contracts: string;
            Sudo: string;
            ImOnline: string;
            __Unused24: string;
            __Unused25: string;
            __Unused26: string;
            __Unused27: string;
            Identity: string;
            Society: string;
            Recovery: string;
            Vesting: string;
            Scheduler: string;
            Glutton: string;
            Preimage: string;
            Proxy: string;
            Multisig: string;
            Bounties: string;
            Tips: string;
            Assets: string;
            PoolAssets: string;
            Beefy: string;
            __Unused42: string;
            __Unused43: string;
            Lottery: string;
            Nis: string;
            Uniques: string;
            Nfts: string;
            NftFractionalization: string;
            Salary: string;
            CoreFellowship: string;
            TransactionStorage: string;
            VoterList: string;
            StateTrieMigration: string;
            ChildBounties: string;
            Referenda: string;
            Remark: string;
            __Unused57: string;
            ConvictionVoting: string;
            Whitelist: string;
            AllianceMotion: string;
            Alliance: string;
            NominationPools: string;
            RankedPolls: string;
            RankedCollective: string;
            AssetConversion: string;
            FastUnstake: string;
            MessageQueue: string;
            __Unused68: string;
            TxPause: string;
            SafeMode: string;
            __Unused71: string;
            MultiBlockMigrations: string;
            Broker: string;
            TasksExample: string;
            __Unused75: string;
            __Unused76: string;
            __Unused77: string;
            __Unused78: string;
            AssetConversionMigration: string;
            Revive: string;
            __Unused81: string;
            DelegatedStaking: string;
            AssetRewards: string;
            AssetsFreezer: string;
            __Unused85: string;
            __Unused86: string;
            __Unused87: string;
            __Unused88: string;
            MetaTx: string;
        };
    };
};
export default _default;
