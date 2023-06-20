#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod escrow {
    use ink::storage::Mapping;
    use openbrush::traits::String;
    // use openbrush::traits::Balance;

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    // #[ink::storage_item]
    pub struct PaymentInfo {
        pub patron: AccountId,
        pub auditor: AccountId,
        pub value: Balance,
        pub aribterprovider: AccountId,
        pub deadline: u64,
        pub inheritedfrom: u32,
        pub creationtime: u64,
        pub completed: bool,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    // #[ink::storage_item]
    pub struct IncreaseRequest {
        haircut_percentage: Balance,
        newdeadline: u64,
    }
    // patron: AccountId,
    //removed unnecessary AccountId param from IncreaseRequest, as we only have to verify the auditor for request.

    #[ink(event)]
    pub struct AuditIdAssigned {
        id: Option<u32>,
        payment_info: Option<PaymentInfo>,
    }
    #[ink(event)]
    pub struct AuditInfoUpdated {
        id: Option<u32>,
        payment_info: Option<PaymentInfo>,
        updated_by: Option<AccountId>,
    }
    #[ink(event)]
    pub struct DeadlineExtendRequest {
        id: u32,
        newtime: u64,
        haircut: Balance,
    }

    #[ink(event)]
    pub struct AuditSubmitted {
        id: u32,
        ipfs_hash: String,
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Escrow {
        current_audit_id: u32,
        audit_id_to_payment_info: Mapping<u32, PaymentInfo>,
        // for the transfers of funds to the arbiters, we will simply send the money to arbiterprovider in the withdrawal, and they are supposed to deal with the rest.
        audit_id_to_time_increase_request: ink::storage::Mapping<u32, IncreaseRequest>,
        audit_id_to_ipfs_hash: ink::storage::Mapping<u32, String>,
    }

    impl Escrow {
        #[ink(constructor)]
        pub fn new() -> Self {
            let current_audit_id = u32::default();
            // let current_request_id = u32::default();
            let audit_id_to_payment_info = Mapping::default();
            let audit_id_to_time_increase_request = Mapping::default();
            let audit_id_to_ipfs_hash = Mapping::default();
            Self {
                current_audit_id,
                audit_id_to_payment_info,
                audit_id_to_time_increase_request,
                audit_id_to_ipfs_hash,
            }
        }

        #[ink(message)]
        pub fn get_current_audit_id(&self) -> u32 {
            self.current_audit_id
        }

        #[ink(message)]
        pub fn get_paymentinfo(&self, id: u32) -> Option<PaymentInfo> {
            self.audit_id_to_payment_info.get(&id)
        }

        #[ink(message)]
        pub fn query_timeincreaserequest(&self, id: u32) -> Option<IncreaseRequest> {
            self.audit_id_to_time_increase_request.get(&id)
        }

        #[ink(message)]
        pub fn create_new_payment(
            &mut self,
            _value: Balance,
            _auditor: AccountId,
            _arbiter_provider: AccountId,
            _deadline: u64,
        ) -> bool {
            let _now = self.env().block_timestamp();
            let x = PaymentInfo {
                value: _value,
                creationtime: _now,
                auditor: _auditor,
                aribterprovider: _arbiter_provider,
                patron: self.env().caller(),
                deadline: _deadline,
                inheritedfrom: 0,
                completed: false,
            };
            //condition to check that the audit is for more than 0 amount.
            if _value == 0 {
                return false;
            }

            self.audit_id_to_payment_info
                .insert(&self.current_audit_id, &x);
            self.env().emit_event(AuditIdAssigned {
                id: Some(self.current_audit_id),
                payment_info: Some(x),
            });
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
        pub fn request_additional_time(
            &mut self,
            _id: u32,
            _time: u64,
            _haircut_percentage: Balance,
        ) -> bool {
            if self.get_payment_info(&_id).unwrap().auditor == self.env().caller() {
                let x = IncreaseRequest {
                    haircut_percentage: _haircut_percentage,
                    newdeadline: _time,
                };
                self.audit_id_to_time_increase_request.insert(_id, &x);
                self.env().emit_event(DeadlineExtendRequest {
                    id: _id,
                    newtime: _time,
                    haircut: _haircut_percentage,
                });
                return true;
            }
            false
        }

        #[ink(message)]
        pub fn approve_additional_time(&mut self, id: u32) -> bool {
            if self.get_payment_info(&id).unwrap().patron == self.env().caller() {
                let haircut = self
                    .query_timeincreaserequest(id)
                    .unwrap()
                    .haircut_percentage;
                if haircut < 100 {
                    let new_deadline = self.query_timeincreaserequest(id).unwrap().newdeadline;
                    //if *= doesn't work, use "self.get_payment_info(&id).unwrap().value *"
                    self.audit_id_to_payment_info.get(id).unwrap().value *= (100 - haircut) / 100;
                    self.audit_id_to_payment_info.get(id).unwrap().deadline = new_deadline;
                    self.env().emit_event(AuditInfoUpdated {
                        id: Some(id),
                        payment_info: Some(self.audit_id_to_payment_info.get(id).unwrap()),
                        updated_by: Some(self.get_payment_info(&id).unwrap().patron),
                    });
                    return true;
                }
                return false;
            }
            false
        }

        #[ink(message)]
        pub fn mark_completed(&mut self, id: u32, _ipfs_hash: String) -> bool {
            if self.get_payment_info(&id).unwrap().auditor == self.env().caller() {
                self.audit_id_to_ipfs_hash.insert(id, &_ipfs_hash);
                self.env().emit_event(AuditSubmitted {
                    id: id,
                    ipfs_hash: _ipfs_hash,
                });
                return true;
            }
            false
        }

        #[ink(message)]
        pub fn assess_audit(&mut self, _id: u32, answer: bool) {
            let caller = self.env().caller();
            let audit_patron = self.audit_id_to_payment_info.get(_id).unwrap().patron;
            let audit_arbiterprovider = self
                .audit_id_to_payment_info
                .get(_id)
                .unwrap()
                .aribterprovider;
            if caller == audit_patron || caller == audit_arbiterprovider {
                if answer && !self.audit_id_to_payment_info.get(_id).unwrap().completed {
                    if caller == audit_patron {
                        self.audit_id_to_payment_info.get(_id).unwrap().completed = answer;
                    // to_do
                    //transferring tokens 98% to the auditor,
                    //transferring tokens 2% to the arbiter_provider for being there.
                    } else {
                        self.audit_id_to_payment_info.get(_id).unwrap().completed = answer;
                        //to_do
                        //transfer 85% to the auditor, 15 for arbiterprovider and 5 arbiters.
                    }
                } else {
                    if caller == audit_patron {
                        //to_do
                        //fire an event for the backend to assign the arbiters on this project.
                    } else {
                        //extend deadline by calling.
                        //also check for polymorphism in this code.
                    }
                }
            }
        }

        #[ink(message)]
        pub fn arbiters_extend_deadline(&self, _id: u32, newdeadline: u64, haircut: Balance) -> bool {
            //checking for the haircut to be lesser than 10% and new deadline to be at least more than 1 day.
            if haircut < 10 && newdeadline > self.env().block_timestamp() + 86400 {
                self.audit_id_to_payment_info.get(_id).unwrap().deadline = newdeadline;
                self.audit_id_to_payment_info.get(_id).unwrap().value *= (100-haircut)/100;
                self.env().emit_event(AuditInfoUpdated {
                    id: Some(_id),
                    payment_info: Some(self.audit_id_to_payment_info.get(_id).unwrap()),
                    updated_by: Some(self.get_payment_info(&_id).unwrap().patron),
                });
                return true;
            }
            false
        }
    }
}
