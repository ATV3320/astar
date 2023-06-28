#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contractCalling {

    use core::hash;

    #[ink(storage)]
    pub struct ContractCalling {
        value: bool,
        number: u32
    }

    impl ContractCalling {
        #[ink(constructor)]
        pub fn new() -> Self {

            Self { value: true,
            	   number: 15 }
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

    }

}
