const unifyMetadata = (metadata) => {
  if ("magicNumber" in metadata) metadata = metadata.metadata;
  if ("tag" in metadata) {
    if (metadata.tag !== "v14" && metadata.tag !== "v15" && metadata.tag !== "v16")
      throw new Error("Only metadata 14, 15, and 16 are supported");
    metadata = metadata.value;
  }
  if ("signedExtensionsByVersion" in metadata.extrinsic) {
    return { version: 16, ...metadata };
  }
  if ("custom" in metadata) {
    const { lookup: lookup2, extrinsic: extrinsic2, custom, apis, pallets: pallets2, outerEnums } = metadata;
    return {
      version: 15,
      lookup: lookup2,
      pallets: pallets2.map((p) => ({
        ...p,
        calls: p.calls != null ? { type: p.calls } : void 0,
        events: p.events != null ? { type: p.events } : void 0,
        errors: p.errors != null ? { type: p.errors } : void 0,
        viewFns: [],
        associatedTypes: []
      })),
      extrinsic: { ...extrinsic2, version: [extrinsic2.version] },
      apis,
      outerEnums,
      custom
    };
  }
  const { lookup, extrinsic, pallets } = metadata;
  return {
    version: 14,
    lookup,
    pallets: pallets.map((p) => ({
      ...p,
      calls: p.calls != null ? { type: p.calls } : void 0,
      events: p.events != null ? { type: p.events } : void 0,
      errors: p.errors != null ? { type: p.errors } : void 0,
      viewFns: [],
      associatedTypes: []
    })),
    extrinsic: { ...extrinsic, version: [extrinsic.version] },
    apis: []
  };
};

export { unifyMetadata };
//# sourceMappingURL=unified.mjs.map
