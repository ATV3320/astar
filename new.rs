#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod escrow {
    use ink::{storage::Mapping};
    // use openbrush::traits::Balance;

    #[derive(scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
    // #[ink::storage_item]
    pub struct PaymentInfo {
     patron: AccountId,
     auditor: AccountId,
     value: Balance,
     aribterprovider: AccountId,
     deadline: u64,
     inheritedfrom: u32,
     creationtime: u64
    }
    
    #[derive(scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
    // #[ink::storage_item]
    pub struct IncreaseRequest {
        haircut_percentage: Balance,
        newdeadline: u64
    }
    // patron: AccountId,
    //removed unnecessary AccountId param from IncreaseRequest, as we only have to verify the auditor for request.

    #[ink(storage)]
    #[derive(Default)]
    pub struct Escrow {
        current_audit_id: u32,
        audit_id_to_payment_info: Mapping<u32, PaymentInfo>,
    // for the transfers of funds to the arbiters, we will simply send the money to arbiterprovider in the withdrawal, and they are supposed to deal with the rest.
        audit_id_to_time_increase_request: ink::storage::Mapping<u32, IncreaseRequest>,

    }

    impl Escrow {

        #[ink(constructor)]
        pub fn new() -> Self {
            let current_audit_id = u32::default();
            // let current_request_id = u32::default();
            let audit_id_to_payment_info = Mapping::default();
            let audit_id_to_time_increase_request = Mapping::default();
            Self {  current_audit_id, audit_id_to_payment_info, audit_id_to_time_increase_request}
        }

        #[ink(message)]
        pub fn get_current_audit_id(&self) -> u32{
            self.current_audit_id
        }

        #[ink(message)]
        pub fn get_paymentinfo(&self, id: u32) -> Option<PaymentInfo> {
            self.audit_id_to_payment_info.get(&id)
        }

        #[ink(message)]
        pub fn query_timeincreaserequest(&self, id:u32) -> Option<IncreaseRequest> {
            self.audit_id_to_time_increase_request.get(&id)
        }

        #[ink(message)]
        pub fn create_new_payment(&mut self, _value:Balance, _auditor:AccountId, _arbiter_provider: AccountId, _deadline: u64 ) -> bool {
            let _now =  self.env().block_timestamp();
            let x = PaymentInfo{
                value: _value,
                creationtime: _now,
                auditor: _auditor,
                aribterprovider: _arbiter_provider,
                patron: self.env().caller(),
                deadline: _deadline,
                inheritedfrom: 0

            };
            //condition to check that the audit is for more than 0 amount.
            if _value==0 {
                return false;
            }

            self.audit_id_to_payment_info.insert(&self.current_audit_id, &x);
            self.current_audit_id = self.current_audit_id + 1;
            true
        }

        #[ink(message)]
        pub fn view_payment_info(&self, id: u32) -> Option<PaymentInfo> {
            self.get_payment_info(&id)
        }

        #[inline]
        fn get_payment_info(&self, id: &u32) -> Option<PaymentInfo> {
            self.audit_id_to_payment_info.get(id)
        }

        #[ink(message)]
        pub fn request_additional_time(&mut self, id: u32, _time: u64, _haircut_percentage: Balance) -> bool {
            if self.get_payment_info(&id).unwrap().auditor == self.env().caller() {
                let x = IncreaseRequest{
                    haircut_percentage : _haircut_percentage,
                    newdeadline: _time
                };
                self.audit_id_to_time_increase_request.insert(id, &x);
                return true;
            }
            false
        }

        #[ink(message)]
        pub fn approve_additional_time(&mut self, id:u32) -> bool {
            if self.get_payment_info(&id).unwrap().patron == self.env().caller() {
                let haircut = self.query_timeincreaserequest(id).unwrap().haircut_percentage;
                if haircut<100 {
                    let mut updated_payment_info = self.get_payment_info(&id).unwrap();
                let new_value = updated_payment_info.value * (100-haircut)/100;
                let new_deadline = self.query_timeincreaserequest(id).unwrap().newdeadline;
                updated_payment_info.deadline = new_deadline;
                updated_payment_info.value = new_value;
                self.audit_id_to_payment_info.insert(id, &updated_payment_info);
                return true;
                }
                return false;
            }
            false
        }

        #[ink(message)]
        pub fn withdraw(&mut self, id:u32) -> bool {
            
        }
    }
}