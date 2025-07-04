use babe_primitives::BABE_ENGINE_ID;
use sc_consensus::BlockImportParams;
use sp_consensus_aura::AURA_ENGINE_ID;
use sp_runtime::ConsensusEngineId;
use sp_runtime::DigestItem;
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};

/// Returns the ConsensusEngineId of the block based on its digest.
///
/// Panics if seal is not Aura or Babe.
pub fn block_consensus_engine_id<Block: BlockT>(
    block: &BlockImportParams<Block>,
) -> ConsensusEngineId {
    let is_aura = block
        .header
        .digest()
        .log(|d| match d {
            DigestItem::PreRuntime(engine_id, _) => {
                if engine_id.clone() == AURA_ENGINE_ID {
                    Some(d)
                } else {
                    None
                }
            }
            _ => None,
        })
        .is_some();
    let is_babe = block
        .header
        .digest()
        .log(|d| match d {
            DigestItem::PreRuntime(engine_id, _) => {
                if engine_id.clone() == BABE_ENGINE_ID {
                    Some(d)
                } else {
                    None
                }
            }
            _ => None,
        })
        .is_some();
    if is_aura {
        AURA_ENGINE_ID
    } else if is_babe {
        BABE_ENGINE_ID
    } else {
        panic!(
            "Unexpected consensus engine ID in block: {:?}",
            block.header.digest()
        );
    }
}
