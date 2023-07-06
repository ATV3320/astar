#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod Token {
    use ink::storage::Mapping;
    use scale::{Decode,Encode};

    /// A simple ERC-20 contract.
    #[ink(storage)]
    //#[derive(Default)]
    pub struct Token {
        /// Total token supply.
        total_supply: Balance,
        /// Mapping from owner to number of owned token.
        balances: Mapping<AccountId, Balance>,
        /// Owner of the contract
        owner: AccountId,
    }

    #[derive(Debug, PartialEq, Eq, Encode, Decode, Clone, Copy)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        UnAuthorisedCall,
    }
    

    /// The ERC-20 result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Token {
        /// Creates a new ERC-20 contract with the specified initial supply.
        #[ink(constructor)]
        pub fn new(owner: AccountId) -> Self {
            Self {
                owner,
                total_supply : Default::default(),
                balances : Mapping::default(),
            }
        }

        // Mints token to a particular address
        #[ink(message)]
        pub fn mint(&mut self, owner: AccountId, amount: Balance)-> Result<()> {
            let caller= self.env().caller();
            if self.owner != caller {
                return Err(Error::UnAuthorisedCall);
            }
            self.balances.insert(owner, &amount);
            Ok(())
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_impl(&owner)
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        ///
        /// # Note
        ///
        /// Prefer to call this method over `balance_of` since this
        /// works using references which are more efficient in Wasm.
        #[inline]
        fn balance_of_impl(&self, owner: &AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

    }
    }