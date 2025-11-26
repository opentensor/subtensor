import type { Collective, HasProposalsFnRet, ProposalCountFn, ProposalFnRet, ProposalHashesFn, ProposalsFnRet } from './types.js';
export declare function hasProposals(section: Collective): HasProposalsFnRet;
export declare function proposals(section: Collective): ProposalsFnRet;
export declare function proposal(section: Collective): ProposalFnRet;
export declare const proposalCount: ProposalCountFn;
export declare const proposalHashes: ProposalHashesFn;
