declare const _default: {
    AuctionIndex: string;
    LeasePeriod: string;
    LeasePeriodOf: string;
    SlotRange10: {
        _enum: string[];
    };
    SlotRange: {
        _enum: string[];
    };
    WinningData10: string;
    WinningData: string;
    WinningDataEntry: string;
    WinnersData10: string;
    WinnersData: string;
    WinnersDataTuple10: string;
    WinnersDataTuple: string;
    Bidder: {
        _enum: {
            New: string;
            Existing: string;
        };
    };
    IncomingParachain: {
        _enum: {
            Unset: string;
            Fixed: string;
            Deploy: string;
        };
    };
    IncomingParachainDeploy: {
        code: string;
        initialHeadData: string;
    };
    IncomingParachainFixed: {
        codeHash: string;
        codeSize: string;
        initialHeadData: string;
    };
    NewBidder: {
        who: string;
        sub: string;
    };
    SubId: string;
};
export default _default;
