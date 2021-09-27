use crate::{
    collections::TypedUsize,
    gg20::crypto_tools::{constants, hash, k256_serde, paillier, vss},
    sdk::{
        api::TofnResult,
        implementer_api::{serialize, ProtocolBuilder, RoundBuilder},
    },
};
use serde::{Deserialize, Serialize};

use super::{
    r2, KeygenPartyShareCounts, KeygenProtocolBuilder, KeygenShareId, PartyKeyPair, PartyZkSetup,
};

#[cfg(feature = "malicious")]
use super::malicious::Behaviour;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Bcast {
    pub(super) y_i_commit: hash::Output,
    pub(super) ek: paillier::EncryptionKey,
    pub(super) ek_proof: paillier::zk::EncryptionKeyProof,
    pub(super) zkp: paillier::zk::ZkSetup,
    pub(super) zkp_proof: paillier::zk::ZkSetupProof,
}

pub(super) fn start(
    my_keygen_id: TypedUsize<KeygenShareId>,
    threshold: usize,
    party_share_counts: KeygenPartyShareCounts,
    keypair: &PartyKeyPair,
    zksetup: &PartyZkSetup,
    #[cfg(feature = "malicious")] behaviour: Behaviour,
) -> TofnResult<KeygenProtocolBuilder> {
    let u_i_vss = vss::Vss::new(threshold);

    let (y_i_commit, y_i_reveal) = hash::commit(
        constants::Y_I_COMMIT_TAG,
        my_keygen_id,
        k256_serde::to_bytes(&(k256::ProjectivePoint::generator() * u_i_vss.get_secret())),
    );
    corrupt!(
        y_i_commit,
        malicious::corrupt_commit(my_keygen_id, &behaviour, y_i_commit)
    );

    let ek_proof = keypair.ek.correctness_proof(
        &keypair.dk,
        &party_share_counts
            .share_to_party_id(my_keygen_id)?
            .to_bytes(),
    );
    corrupt!(
        ek_proof,
        malicious::corrupt_ek_proof(my_keygen_id, &behaviour, ek_proof)
    );

    let zkp_proof = zksetup.zkp_proof.clone();
    corrupt!(
        zkp_proof,
        malicious::corrupt_zkp_proof(my_keygen_id, &behaviour, zkp_proof)
    );

    let bcast_out = Some(serialize(&Bcast {
        y_i_commit,
        ek: keypair.ek.clone(),
        ek_proof,
        zkp: zksetup.zkp.clone(),
        zkp_proof,
    })?);

    Ok(ProtocolBuilder::NotDone(RoundBuilder::new(
        Box::new(r2::R2 {
            threshold,
            party_share_counts,
            dk: keypair.dk.clone(),
            u_i_vss,
            y_i_reveal,
            #[cfg(feature = "malicious")]
            behaviour,
        }),
        bcast_out,
        None,
    )))
}

#[cfg(feature = "malicious")]
mod malicious {
    use crate::{
        collections::TypedUsize,
        gg20::{
            crypto_tools::{
                hash::Output,
                paillier,
                paillier::zk::{EncryptionKeyProof, ZkSetupProof},
            },
            keygen::{malicious::Behaviour, KeygenShareId},
        },
    };
    use tracing::info;

    pub fn corrupt_commit(
        my_keygen_id: TypedUsize<KeygenShareId>,
        behaviour: &Behaviour,
        commit: Output,
    ) -> Output {
        if let Behaviour::R1BadCommit = behaviour {
            info!("malicious peer {} does {:?}", my_keygen_id, behaviour);
            commit.corrupt()
        } else {
            commit
        }
    }

    pub fn corrupt_ek_proof(
        my_keygen_id: TypedUsize<KeygenShareId>,
        behaviour: &Behaviour,
        ek_proof: EncryptionKeyProof,
    ) -> EncryptionKeyProof {
        if let Behaviour::R1BadEncryptionKeyProof = behaviour {
            info!("malicious peer {} does {:?}", my_keygen_id, behaviour);
            paillier::zk::malicious::corrupt_ek_proof(ek_proof)
        } else {
            ek_proof
        }
    }

    pub fn corrupt_zkp_proof(
        my_keygen_id: TypedUsize<KeygenShareId>,
        behaviour: &Behaviour,
        zkp_proof: ZkSetupProof,
    ) -> ZkSetupProof {
        if let Behaviour::R1BadZkSetupProof = behaviour {
            info!("malicious peer {} does {:?}", my_keygen_id, behaviour);
            paillier::zk::malicious::corrupt_zksetup_proof(zkp_proof)
        } else {
            zkp_proof
        }
    }
}
