local m = import './spec.libsonnet';
local {encodeGrandpaKeys} = import '../util/grandpaKeys.libsonnet';
local strToHex(str) = cql.toHex(std.encodeUTF8(str));
local
	account(name) = cql.sr25519Seed(name),
	unwrapNewtype(struct) = local names = std.objectFields(struct);
		if std.length(names) == 1 then struct[names[0]]
		else struct,
	WELLKNOWN_CODE = strToHex(':code'),
	WELLKNOWN_GRANDPA_AUTHORITIES = strToHex(':grandpa_authorities'),
;

{
	resetSystem: function(prev) prev {
		_storage+: {
			System+: {
				// Magic value mandated by spec, nice
				// [69, 69, 69, ..., 69, 69, 69]
				local hash69 = '0x' + '45' * 32,
				BlockHash: {
					"0": hash69,
				},
				EventCount: 0,
				EventTopics: {},
				Events: [],
				ExtrinsicCount: 0,
				ExtrinsicData: {},

				// Block header
				ParentHash: hash69,
				Number: 0,
				Digest: [],
			},
			Aura+: {
				CurrentSlot: std.bigint('0'),
			},
			[if 'CollatorSelection' in prev._storage then 'CollatorSelection']+: {
				LastAuthoredBlock: {},
			},
			[if 'Session' in prev._storage then 'Session']+: {
				CurrentIndex: 0,
			},
			// Full reset
			[if 'ParachainSystem' in prev._storage then 'ParachainSystem']: {},
			[if 'Authorship' in prev._storage then 'Authorship']: {},
		},
	},

	setSudo(key): {
		_storage+: {
			Sudo+: {
				Key: key,
			},
		},
	},
	giveBalance(address, amount): {
		_storage+: {
			// Not updating total issuance: no big difference.
			System+: {
				Account+: std.trace('Altering account %s: %s' % [address, super.Account?.[address] ?? '<new>'], {
					[address]+: {
						nonce: super?.nonce ?? 0,
						// Leaks
						consumers: super?.consumers ?? 1,
						providers: super?.providers ?? 1,
						sufficients: super?.sufficients ?? 0,
						data+: {
							free+: std.bigint(amount),
							reserved: super?.reserved ?? std.bigint('0'),
							misc_frozen: super?.misc_frozen ?? std.bigint('0'),
							fee_frozen: super?.fee_frozen ?? std.bigint('0'),
						},
					},
				}),
			},
		},
	},
	setParaId(id): function(prev) prev {
		// COMPAT: unique-chain
		[if 'para_id' in prev then 'para_id']: id,
		_storage+: {
			[if 'ParachainInfo' in prev._storage then 'ParachainInfo']: {
				ParachainId: id,
			},
		},
	},
	resetAuraAuthorities: function(prev) prev {
		_storage+: {
			Aura+: {
				Authorities: [],
			},
			[if 'AuraExt' in prev._storage then 'AuraExt']+: {
				Authorities: [],
			},
		},
	},
	addAuraAuthority(key): function(prev) prev {
		_storage+: {
			Aura+: {
				Authorities+: [cql.ss58(key)],
			},
			[if 'AuraExt' in prev._storage then 'AuraExt']+: {
				Authorities+: [cql.ss58(key)],
			},
		},
	},
	setGrandpaKeys(keys): function(prev) prev {
		_storage+: {
			_unknown+: {
				[if WELLKNOWN_GRANDPA_AUTHORITIES in prev._storage._unknown then WELLKNOWN_GRANDPA_AUTHORITIES]: encodeGrandpaKeys(keys),
			},
		},
	},
	resetSessionKeys: {
		_storage+: {
			Session+: {
				DisabledValidators: [],
				KeyOwner: {},
				NextKeys: {},
				QueuedKeys: [],
				Validators: [],
			},
		},
	},
	addSessionKey([_accountId, _validatorId, _keys]): {
		local accountId = cql.ss58(_accountId),
		local validatorId = cql.ss58(_validatorId),
		local keys = {
			[cql.toHex(std.encodeUTF8(key))]: cql.ss58(data),
			for [key, data] in _keys
		},
		_storage+: {
			// FIXME: Should increase consumers/providers for account
			Session+: {
				KeyOwner+: {
					[std.toString([key, data])]: validatorId,
					for [key, data] in keys
				},
				NextKeys+: {
					[validatorId]: unwrapNewtype(keys),
				},
				QueuedKeys+: [
					[
						validatorId,
						unwrapNewtype(keys),
					],
				],
				Validators+: [validatorId],
			},
		},
	},
	resetInvulnerables: function(prev) prev {
		_storage+: {
			[if 'CollatorSelection' in prev._storage then 'CollatorSelection']+: {
				Invulnerables: [],
			},
		}
	},
	addInvulnerable(key): function(prev) prev {
		_storage+: {
			[if 'CollatorSelection' in prev._storage then 'CollatorSelection']+: {
				Invulnerables+: [cql.ss58(key)],
			},
		}
	},

	setCodeRaw(code): function(prev) prev {
		genesis+: {
			raw+: {
				top+: {
					[WELLKNOWN_CODE]: code,
				},
			},
		},
	},

	// Compatible, as storage remains the same
	resetNetworking: m.resetNetworking,

	decodeSpec(): function(prev) local dump = cql.fullDump(prev.genesis.raw.top); prev {
		_originalDump:: dump,
		_storage::: dump,
		genesis+: {
			raw+: {
				top:: error "reencode storage first"
			},
		},
	},
	reencodeSpec(): function(prev) prev {
		_originalDump:: error "decode storage first",
		_storage:: error "decode storage first",
		genesis+: {
			raw+: {
				top::: prev._originalDump._rebuild(prev._storage),
			},
		},
	},

	polkaLaunchPara(root): [
		$.resetSystem,
		// $.setSudo(account('//Alice')),
		// Will break everything
		// $.resetBalances,
		$.resetAuraAuthorities,
		[
			$.addAuraAuthority(node.keys.aura),
			for [?, node] in root.nodes
		],
		$.setGrandpaKeys([node.keys.gran for [?, node] in root.nodes]),
		function(prev) bdk.mixer(if 'Session' in prev._storage then [
			$.resetSessionKeys,
			[
				$.addSessionKey([
					node.keys.aura,
					node.keys.aura,
					{
						aura: node.keys.aura,
					},
				]),
				for [?, node] in root.nodes
			],
		] else [])(prev),
		$.resetInvulnerables,
		[
			$.addInvulnerable(node.keys.aura),
			for [?, node] in root.nodes
		],
		$.setParaId(root.paraId),
	],
}
