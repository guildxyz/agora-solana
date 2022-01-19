use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

pub type SignerPdaError = &'static str;

/// PDA with easy access to its signer seeds.
#[derive(Debug, Clone, Copy)]
pub struct SignerPda<'a, 'b> {
    pub pda: Pubkey,
    pub bump: [u8; 1],
    pub seeds: &'b [&'a [u8]],
}

impl<'a, 'b> SignerPda<'a, 'b> {
    /// Computes a new PDA and checks whether it matches the expected address.
    pub fn new_checked(
        seeds: &'b [&'a [u8]],
        program_id: &Pubkey,
        expected: &AccountInfo,
    ) -> Result<Self, ProgramError> {
        let (pda, bump) = Self::find_and_check(seeds, program_id, expected.key)?;
        Ok(Self {
            pda,
            bump: [bump],
            seeds,
        })
    }

    /// Checks whether there's an existing PDA account with the program as its
    /// owner.
    pub fn check_existing(
        seeds: &'b [&'a [u8]],
        program_id: &Pubkey,
        expected: &AccountInfo,
    ) -> Result<(), ProgramError> {
        Self::find_and_check(seeds, program_id, expected.key)?;
        if expected.owner != program_id {
            Err(ProgramError::IllegalOwner)
        } else {
            Ok(())
        }
    }

    fn find_and_check(
        seeds: &'b [&'a [u8]],
        program_id: &Pubkey,
        expected: &Pubkey,
    ) -> Result<(Pubkey, u8), ProgramError> {
        let (pda, bump) = Pubkey::find_program_address(seeds, program_id);
        if &pda != expected {
            Err(ProgramError::InvalidSeeds)
        } else {
            Ok((pda, bump))
        }
    }

    /// Returns the signer seeds (seeds + bump seed) of the PDA.
    pub fn signer_seeds(&'a self) -> Vec<&'a [u8]> {
        let mut signer_seeds = self.seeds.to_vec();
        signer_seeds.push(&self.bump);
        signer_seeds
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_checks_and_seeds() {
        let program_id = Pubkey::new_unique();
        let seed_pubkey = Pubkey::new_unique();
        let seeds = &[b"this is a seed", seed_pubkey.as_ref()];
        let (pda, bump) = Pubkey::find_program_address(seeds, &program_id);

        let mut data = [2_u8, 3, 4, 5, 6, 7];
        let mut lamports = 1500;
        let mut account_info = AccountInfo::new(
            &pda,
            false,
            false,
            &mut lamports,
            data.as_mut_slice(),
            &program_id,
            false,
            0,
        );
        let signer_pda = SignerPda::new_checked(seeds, &program_id, &account_info).unwrap();
        assert_eq!(signer_pda.pda, pda);
        let mut expected_signer_seeds = seeds.to_vec();
        let bump_slice = [bump];
        expected_signer_seeds.push(&bump_slice);
        assert_eq!(signer_pda.signer_seeds(), expected_signer_seeds);

        // bad program_id
        assert_eq!(
            SignerPda::new_checked(seeds, &Pubkey::new_unique(), &account_info)
                .err()
                .unwrap(),
            ProgramError::InvalidSeeds
        );
        // bad seeeds
        assert_eq!(
            SignerPda::new_checked(&[b"bad seed"], &program_id, &account_info)
                .err()
                .unwrap(),
            ProgramError::InvalidSeeds
        );
        // check existing
        assert!(SignerPda::check_existing(seeds, &program_id, &account_info).is_ok());
        // bad owner
        let new_owner = Pubkey::new_unique();
        account_info.owner = &new_owner;
        assert_eq!(
            SignerPda::check_existing(seeds, &program_id, &account_info)
                .err()
                .unwrap(),
            ProgramError::IllegalOwner
        );
    }
}
