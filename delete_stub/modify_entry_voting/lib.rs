#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod modify {
    pub type Result<T> = core::result::Result<T, Error>;
    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Error {
        UnAuthorisedCall,
        AssessmentFailed,
        ResultAlreadyPublished,
        VotingFailed,
        RightsNotActivatedYet,
        TransferFailed,
        TreasuryEmpty,
        NoOneVoted,
        ValueTooLow,
        ValueTooHigh,
    }
    #[ink(storage)]
    pub struct Modify {
        pub current_id: u32,
    }
    impl Modify {
        #[ink(constructor)]
        pub fn new(id: u32) -> Self {
            let current_id = id;
            Self { current_id }
        }

        #[ink(message)]
        pub fn modify_value(&self, target: AccountId, where1: u32, what: Balance) -> bool{
            let _modify_registry_in_voting = ink::env::call::build_call::<Environment>()
                .call(target)
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                        ink::selector_bytes!("add_to_treasury"),
                    ))
                    .push_arg(where1)
                    .push_arg(what), // .push_arg(&[0x10u8; 32]),
                )
                .returns::<Result<()>>()
                .try_invoke();
            true
        }
    }
}
