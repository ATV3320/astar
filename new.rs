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
     notbefore: u64,
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
        auditid: u32,
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
            let audit_id_to_payment_info = Mapping::default();
            let audit_id_to_time_increase_request = Mapping::default();
            Self {  current_audit_id, audit_id_to_payment_info, audit_id_to_time_increase_request}
        }

        #[ink(message)]
        pub fn get_current_audit_id(&self) -> u32{
            // let caller = self.env().caller();
            // self.redeemableAmount
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
        pub fn create_new_payment(&mut self, _value:Balance, _auditor:AccountId, _arbiter_provider: AccountId, _notbefore: u64 ) -> bool {
            // let _patron = self.env().caller();
            //won't need it since we're already passing the struct, and it contains the patron's account_id
            let _now =  self.env().block_timestamp();
            let x = PaymentInfo{
                value: _value,
                creationtime: _now,
                auditor: _auditor,
                aribterprovider: _arbiter_provider,
                patron: self.env().caller(),
                notbefore: _notbefore,
                inheritedfrom: 0

            };
            //condition to check that the audit is for more than 0 amount.
            if _value==0 {
                return false;
            }

            self.audit_id_to_payment_info.insert(&self.current_audit_id, &x);
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
        pub fn request_additional_time(&mut self, id: u32, _time: u64) -> bool {
            if self.get_payment_info(&id).unwrap().auditor == self.env().caller() {
                let x = IncreaseRequest{
                    auditid : id,
                    newdeadline: _time
                };
                self.audit_id_to_time_increase_request.insert(id, &x);
                return true;
            }
            false
        }

        #[ink(message)]
        pub fn approve_additional_time(&mut self, )
    }
}