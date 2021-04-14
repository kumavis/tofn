use crate::zkp::pedersen;

use super::{Sign, Status};
use curv::{
    elliptic::curves::traits::{ECPoint, ECScalar},
    FE,
};
use serde::{Deserialize, Serialize};

// round 7

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bcast {
    pub ecdsa_sig_summand: FE,
}
#[derive(Debug)] // do not derive Clone, Serialize, Deserialize
pub(super) struct State {
    pub(super) r: FE,
    pub(super) my_ecdsa_sig_summand: FE,
}

impl Sign {
    pub(super) fn r7(&self) -> (State, Bcast) {
        assert!(matches!(self.status, Status::R6));
        let r5state = self.r5state.as_ref().unwrap();
        let r6state = self.r6state.as_ref().unwrap();

        // checks:
        // * sum of ecdsa_public_key_check (S_i) = ecdsa_public_key as per phase 6 of 2020/540
        // * verify zk proofs
        let mut ecdsa_public_key = r6state.my_ecdsa_public_key_check;
        for (i, participant_index) in self.participant_indices.iter().enumerate() {
            if *participant_index == self.my_secret_key_share.my_index {
                continue;
            }
            let in_r6bcast = self.in_r6bcasts.vec_ref()[i].as_ref().unwrap();
            let in_r3bcast = self.in_r3bcasts.vec_ref()[i].as_ref().unwrap();

            pedersen::verify_wc(
                &pedersen::StatementWc {
                    stmt: pedersen::Statement {
                        commit: &in_r3bcast.nonce_x_keyshare_summand_commit,
                    },
                    msg_g: &in_r6bcast.ecdsa_public_key_check,
                    g: &r5state.ecdsa_randomizer,
                },
                &in_r6bcast.ecdsa_public_key_check_proof_wc,
            )
            .unwrap_or_else(|e| {
                panic!(
                    "party {} says: pedersen proof wc failed to verify for party {} because [{}]",
                    self.my_secret_key_share.my_index, participant_index, e
                )
            });

            ecdsa_public_key = ecdsa_public_key + in_r6bcast.ecdsa_public_key_check;
        }
        assert_eq!(ecdsa_public_key, self.my_secret_key_share.ecdsa_public_key); // TODO panic

        // compute our sig share s_i (aka my_ecdsa_sig_summand) as per phase 7 of 2020/540
        let r1state = self.r1state.as_ref().unwrap();
        let r3state = self.r3state.as_ref().unwrap();
        let r: FE = ECScalar::from(
            &r5state
                .ecdsa_randomizer
                .x_coor()
                .unwrap()
                .mod_floor(&FE::q()),
        );
        let my_ecdsa_sig_summand = self.msg_to_sign * r1state.my_ecdsa_nonce_summand
            + r * r3state.my_nonce_x_keyshare_summand;

        (
            State {
                r,
                my_ecdsa_sig_summand,
            },
            Bcast {
                ecdsa_sig_summand: my_ecdsa_sig_summand,
            },
        )
    }
}