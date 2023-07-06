#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contractCalling {
    // use ink::primitives::AccountId;

    #[ink(storage)]
    pub struct ContractCalling {
        value: bool,
        number: u32,
    }

    impl ContractCalling {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                value: true,
                number: 15,
            }
        }

        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }

        #[ink(message)]
        pub fn for_no_reason_number(&self) -> u32 {
            self.number
        }

        #[ink(message)]
        pub fn get_accountID(&self) -> AccountId {
            self.env().account_id()
        }

        // #[ink(message)]
        // pub fn call_another_contract(&self, address : AccountId, _to: AccountId, _amount: Balance) -> Balance {
        //     return ink::env::call::build_call::<Environment>()
        //     .call(address)
        //     .gas_limit(5000)
        //     .exec_input(
        //     ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(ink::selector_bytes!("transfer"))).push_arg(_to).push_arg(_amount)
        // )
        // .returns::<Balance>()
        // .invoke();
        // }

        #[ink(message)]
        pub fn call_another_contract(&mut self, address : AccountId) {
            return ink::env::call::build_call::<Environment>()
            .call(address)
            .gas_limit(0)
            .exec_input(
            ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(ink::selector_bytes!("flip")))
        )
        .returns::<()>()
        .invoke();
        }

        #[ink(message)]
        pub fn call_flipper(&mut self, target_contract: AccountId) {
            ink::env::call::build_call::<Environment>()
                .call(target_contract)
                .gas_limit(0)
                .transferred_value(10)
                .exec_input(
                    ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(ink::selector_bytes!("flip")))
                        // .push_arg(42u8)
                        // .push_arg(true)
                        // .push_arg(&[0x10u8; 32]),
                )
                .returns::<()>()
                .invoke();
        }
    }
}
