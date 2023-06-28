#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contractCalling {

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
        
        #[ink(message)]
        pub fn call_another_contract(&self, address : AccountId) -> u32 {
            return ink::env::call::build_call::<Environment>()
            .call(address)                              
            .gas_limit(0)
            .exec_input(
            ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(ink::selector_bytes!("for_no_reason_number")))
        )
        .returns::<u32>()
        .invoke();
        }
    }
}