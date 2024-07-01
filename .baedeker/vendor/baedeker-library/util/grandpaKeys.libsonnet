local t = import './meta.libsonnet';

local types = t.metadata({
    keys: t.s({
        unused: $.u8,
        list: $.authorityList,
    }),
    authorityList: t.v($.authority),
    authorityId: t.a($.u8, 32),
    authority: t.s({
        id: $.authorityId,
        weight: $.u64,
    }),

    u8: t.p('u8'),
    u64: t.p('u64'),
});

{
    encodeGrandpaKeys(keys): types._encode(0, std.trace({
        unused: 1,
        list: [{id: cql.ss58(key), weight: '1'} for key in keys],
    })),
}
