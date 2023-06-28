#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contractCalled {
    //use ink::contract::contractCalled;

    #[ink(storage)]
    pub struct contractCalled {
        value: u32,
    }

    impl contractCalled {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { value: 0 }
        }

        #[ink(message)]
        pub fn increament(&mut self) {
            self.value += 1;
        }

        #[ink(message)]
        pub fn get_number(&self) -> u32 {
            let token_address: u8 = "YpSrsuvjnLVasoRMo9RkHahFu99ec1GWLvpUZv6h1uCKbx1"
                .parse()
                .unwrap();
            let my_return_value = ink::env::call::build_call::<ink::env::DefaultEnvironment>()
                .call(AccountId::from([token_address; 32]))
                .gas_limit(0)
                .transferred_value(10)
                .exec_input(ink::env::call::ExecutionInput::new(
                    ink::env::call::Selector::new(ink::selector_bytes!("getNumber")),
                ))
                .returns::<u32>()
                .invoke();

            return my_return_value;
        }
    }
}
