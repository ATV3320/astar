#![cfg_attr(not(feature = "std"), no_std, no_main)]
//how to compare non binary stuff, so comparing two enums that are part of a struct should take place like...
// matches!(first.element, second.element) and this would return true or false according to their match.


#[ink::contract]
mod escrow {
    use ink::storage::Mapping;
    // use ink_e2e::subxt::utils::MultiAddress;
    // use openbrush::traits::String;
    // use openbrush::traits::Balance;
    use ink::prelude::string::String;

    //under audit status,
    //if the audit is created, or expired, the patron can take their money back,
    //once the audit is assigned, it will only go to submitted, then completed, or expired from assigned
    //but patron won't be able to cash it out.
    //audit_completed means that the auditor and corresponding parties have been paid.
    //arbiter provider will take out the arbiters' fee at the time of arbiter_extend_deadline only.
    //final arbiterprovider fee and auditor's payment will be completed later.
    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum AuditStatus {
        AuditCreated,
        AuditAssigned,
        AuditSubmitted,
        AuditAwaitingValidation,
        AuditCompleted,
        AuditExpired,
    }

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
        pub arbiterprovider: AccountId,
        pub deadline: u64,
        pub starttime: u64,
        pub currentstatus: AuditStatus,
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

    #[ink(event)]
    pub struct TokenIncoming {
        id: u32,
    }

    #[ink(event)]
    pub struct TokenOutgoing {
        id: u32,
        receiver: AccountId,
        amount: Balance,
    }

    #[ink(event)]
    pub struct AuditIdRetrieved {
        id: u32,
    }

    // pub enum Error1 {
    //     //when the transferFrom of the currency token returns false
    //     transferFromFailed,
    //     //when the transfer function of the currency token returns false
    //     transferFailed,
    //     //when non-patron tries to access function
    //     onlyPatron,
    //     //when non-auditor tries to access function
    //     onlyAuditor,
    //     //when non-arbiterprovider tries to access function
    //     onlyArbiterProvider,
    //     //when the amount/amount percentage exceeds expected values
    //     exceedingAmount,
    // }

    // result type
    // pub type Result<T> = core::result::Result<T, Error>;

    #[ink(storage)]
    pub struct Escrow {
        current_audit_id: u32,
        stablecoin_address: AccountId,
        pub audit_id_to_payment_info: Mapping<u32, PaymentInfo>,
        // for the transfers of funds to the arbiters, we will simply send the money to arbiterprovider in the withdrawal, and they are supposed to deal with the rest.
        pub audit_id_to_time_increase_request: ink::storage::Mapping<u32, IncreaseRequest>,
        pub audit_id_to_ipfs_hash: ink::storage::Mapping<u32, String>,
    }

    impl Escrow {
        #[ink(constructor)]
        pub fn new(_stablecoin_address: AccountId) -> Self {
            let current_audit_id = u32::default();
            let stablecoin_address = _stablecoin_address;
            // let current_request_id = u32::default();
            let audit_id_to_payment_info = Mapping::default();
            let audit_id_to_time_increase_request = Mapping::default();
            let audit_id_to_ipfs_hash = Mapping::default();
            Self {
                current_audit_id,
                stablecoin_address,
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
        pub fn know_your_stablecoin(&self) -> AccountId {
            self.stablecoin_address
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
            _arbiter_provider: AccountId,
            _deadline: u64,
            //this deadline is deadline that will be added to current time once the audit is assigned to an auditor.
        ) -> bool {
            let _now = self.env().block_timestamp();
            let x = PaymentInfo {
                value: _value,
                starttime: _now,
                auditor: self.env().caller(),
                arbiterprovider: _arbiter_provider,
                patron: self.env().caller(),
                deadline: _deadline,
                currentstatus: AuditStatus::AuditCreated,
            };
            //condition to check that the audit is for more than 0 amount.
            if _value == 0 {
                return false;
            }
            if !ink::env::call::build_call::<Environment>()
                .call(self.stablecoin_address)
                .gas_limit(0)
                .transferred_value(0)
                //check further on this transferred_value
                .exec_input(
                    ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                        ink::selector_bytes!("transfer_from"),
                    ))
                    .push_arg(self.env().caller())
                    .push_arg(self.env().account_id())
                    .push_arg(_value),
                )
                .returns::<bool>()
                .invoke()
            {
                return false;
            }
            self.env().emit_event(TokenIncoming {
                id: self.current_audit_id,
            });
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
        pub fn assign_audit(&mut self, id: u32, _auditor: AccountId) -> bool {
            let mut payment_info = self.audit_id_to_payment_info.get(id).unwrap();
            let _now = self.env().block_timestamp();
            if payment_info.patron == self.env().caller()
                && matches!(payment_info.currentstatus, AuditStatus::AuditCreated)
            {
                payment_info.auditor = _auditor;
                payment_info.starttime = _now;
                payment_info.deadline = payment_info.deadline + _now;
                payment_info.currentstatus = AuditStatus::AuditAssigned;
                self.audit_id_to_payment_info.insert(id, &payment_info);
                return true;
            } else {
                return false;
            }
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

                    let mut payment_info = self.audit_id_to_payment_info.get(id).unwrap();
                    let value0 = payment_info.value * haircut /100;
                    if !ink::env::call::build_call::<Environment>()
                        .call(self.stablecoin_address)
                        .gas_limit(0)
                        .transferred_value(0)
                        .exec_input(
                            ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                                ink::selector_bytes!("transfer"),
                            ))
                            .push_arg(payment_info.patron)
                            .push_arg(value0), // .push_arg(&[0x10u8; 32]),
                        )
                        .returns::<bool>()
                        .invoke()
                    {
                        return false;
                    }
                    self.env().emit_event(TokenOutgoing {
                        id: id,
                        receiver: payment_info.patron,
                        amount: value0,
                    });
                    // Update the value in storage
                    payment_info.value = payment_info.value * (100 - haircut) / 100;
                    // Update the deadline in storage
                    payment_info.deadline = new_deadline;
                    // Set the updated payment_info back to storage
                    self.audit_id_to_payment_info.insert(id, &payment_info);
                    //if *= doesn't work, use "self.get_payment_info(&id).unwrap().value *"

                    // self.audit_id_to_payment_info.get(id).unwrap().value *= (100 - haircut) / 100;
                    // self.audit_id_to_payment_info.get(id).unwrap().deadline = new_deadline;
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
        pub fn mark_submitted(&mut self, id: u32, _ipfs_hash: String) -> bool {
            let mut payment_info = self.audit_id_to_payment_info.get(id).unwrap();
            if payment_info.auditor == self.env().caller()
                && matches!(payment_info.currentstatus, AuditStatus::AuditAssigned)
                && payment_info.deadline > self.env().block_timestamp()
            {
                self.audit_id_to_ipfs_hash.insert(id, &_ipfs_hash);
                self.env().emit_event(AuditSubmitted {
                    id: id,
                    ipfs_hash: _ipfs_hash,
                });
                payment_info.currentstatus = AuditStatus::AuditSubmitted;
                self.audit_id_to_payment_info.insert(id, &payment_info);
                return true;
            }
            false
        }
        #[ink(message)]
        pub fn assess_audit(&mut self, id: u32, answer: bool) -> bool {
            let mut payment_info = self.audit_id_to_payment_info.get(id).unwrap();
            //broken down into three cases,
            //C1: when patron calls,
            //C2: when arbiterprovider calls,
            //C3: when anything else happens
            //C1 has two parts further, patron can only assess the audit if it is in submitted state, if patron
            //says yes, then transfers happen, if no, then state is changed to awaitingValidation.
            //C2 could have had two parts, but polymorphism would have been part of the code then,
            //so if the arbiter calls, they must say true in answer, and state should be awaitingValidation
            //only then will the transfers happen.
            //C1
            if self.env().caller() == payment_info.patron
                && matches!(payment_info.currentstatus, AuditStatus::AuditSubmitted)
            {
                if answer {
                    //payment initiated,
                    if !ink::env::call::build_call::<Environment>()
                        .call(self.stablecoin_address)
                        .gas_limit(0)
                        .transferred_value(0)
                        .exec_input(
                            ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                                ink::selector_bytes!("transfer"),
                            ))
                            .push_arg(payment_info.auditor)
                            .push_arg(payment_info.value * 98 / 100), // .push_arg(&[0x10u8; 32]),
                        )
                        .returns::<bool>()
                        .invoke()
                    {
                        return false;
                    }
                    self.env().emit_event(TokenOutgoing {
                        id: id,
                        receiver: payment_info.auditor,
                        amount: payment_info.value * 98 / 100,
                    });
                    ink::env::call::build_call::<Environment>()
                        .call(self.stablecoin_address)
                        .gas_limit(0)
                        .transferred_value(0)
                        .exec_input(
                            ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                                ink::selector_bytes!("transfer"),
                            ))
                            .push_arg(payment_info.arbiterprovider)
                            .push_arg(payment_info.value * 2 / 100), // .push_arg(&[0x10u8; 32]),
                        )
                        .returns::<bool>()
                        .invoke();
                    self.env().emit_event(TokenOutgoing {
                        id: id,
                        receiver: payment_info.arbiterprovider,
                        amount: payment_info.value * 2 / 100,
                    });
                    payment_info.currentstatus = AuditStatus::AuditCompleted;
                    self.audit_id_to_payment_info.insert(id, &payment_info);
                    return true;
                } else {
                    payment_info.currentstatus = AuditStatus::AuditAwaitingValidation;
                    self.audit_id_to_payment_info.insert(id, &payment_info);
                    //to_do event emit here for backend
                    return true;
                }
            }
            //C2
            else if self.env().caller() == payment_info.arbiterprovider
                && matches!(
                    payment_info.currentstatus,
                    AuditStatus::AuditAwaitingValidation
                )
            {
                if answer
                    && ink::env::call::build_call::<Environment>()
                        .call(self.stablecoin_address)
                        .gas_limit(0)
                        .transferred_value(0)
                        .exec_input(
                            ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                                ink::selector_bytes!("transfer"),
                            ))
                            .push_arg(payment_info.auditor)
                            .push_arg(payment_info.value * 95 / 100), // .push_arg(&[0x10u8; 32]),
                        )
                        .returns::<bool>()
                        .invoke()
                {
                    self.env().emit_event(TokenOutgoing {
                        id: id,
                        receiver: payment_info.auditor,
                        amount: payment_info.value * 95 / 100,
                    });
                    ink::env::call::build_call::<Environment>()
                        .call(self.stablecoin_address)
                        .gas_limit(0)
                        .transferred_value(0)
                        .exec_input(
                            ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                                ink::selector_bytes!("transfer"),
                            ))
                            .push_arg(payment_info.arbiterprovider)
                            .push_arg(payment_info.value * 5 / 100), // .push_arg(&[0x10u8; 32]),
                        )
                        .returns::<bool>()
                        .invoke();
                    self.env().emit_event(TokenOutgoing {
                        id: id,
                        receiver: payment_info.arbiterprovider,
                        amount: payment_info.value * 5 / 100,
                    });
                    payment_info.currentstatus = AuditStatus::AuditCompleted;
                    self.audit_id_to_payment_info.insert(id, &payment_info);
                    return true;
                }
            }
            //C3
            false
        }
        //to_do
        // in every function where haircut is mentioned, return the rest of the amount to the patron.
        //complete this task after integrating psp22/erc20
        #[ink(message)]
        pub fn arbiters_extend_deadline(
            &mut self,
            _id: u32,
            new_deadline: u64,
            haircut: Balance,
            arbitersshare: Balance,
        ) -> bool {
            //the goal of this function is to take out arbiters' cut from the payment_info.value and then
            // set new number as value.

            //checking for the haircut to be lesser than 10% and new deadline to be at least more than 1 day.
            if haircut < 10 && new_deadline > self.env().block_timestamp() + 86400 {
                let mut payment_info = self.audit_id_to_payment_info.get(_id).unwrap();

                let arbitersscut: Balance = payment_info.value * arbitersshare / 100;
                // Update the value in storage
                payment_info.value = payment_info.value * (100 - (arbitersshare + haircut)) / 100;
                // Update the deadline in storage
                payment_info.deadline = new_deadline;
                // Set the updated payment_info back to storage
                ink::env::call::build_call::<Environment>()
                    .call(self.stablecoin_address)
                    .gas_limit(0)
                    .transferred_value(0)
                    .exec_input(
                        ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                            ink::selector_bytes!("transfer"),
                        ))
                        .push_arg(payment_info.arbiterprovider)
                        .push_arg(arbitersscut), // .push_arg(&[0x10u8; 32]),
                    )
                    .returns::<bool>();
                self.env().emit_event(TokenOutgoing {
                    id: _id,
                    receiver: payment_info.arbiterprovider,
                    amount: arbitersscut,
                });
                ink::env::call::build_call::<Environment>()
                    .call(self.stablecoin_address)
                    .gas_limit(0)
                    .transferred_value(0)
                    .exec_input(
                        ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                            ink::selector_bytes!("transfer"),
                        ))
                        .push_arg(payment_info.patron)
                        .push_arg(payment_info.value * haircut / 100), // .push_arg(&[0x10u8; 32]),
                    )
                    .returns::<bool>();
                self.env().emit_event(TokenOutgoing {
                    id: _id,
                    receiver: payment_info.patron,
                    amount: payment_info.value * haircut / 100,
                });
                self.audit_id_to_payment_info.insert(_id, &payment_info);
                self.env().emit_event(AuditInfoUpdated {
                    id: Some(_id),
                    payment_info: Some(self.audit_id_to_payment_info.get(_id).unwrap()),
                    updated_by: Some(self.get_payment_info(&_id).unwrap().patron),
                });
                return true;
            }
            false
        }
        pub fn retrieve_audit(&mut self, id: u32) -> bool {
            let mut payment_info = self.audit_id_to_payment_info.get(id).unwrap();
            if payment_info.patron == self.env().caller() && (matches!(payment_info.currentstatus, AuditStatus::AuditCreated) || payment_info.deadline <= self.env().block_timestamp()) {
                payment_info.currentstatus = AuditStatus::AuditExpired;
                payment_info.value = 0;
                ink::env::call::build_call::<Environment>()
                    .call(self.stablecoin_address)
                    .gas_limit(0)
                    .transferred_value(0)
                    .exec_input(
                        ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                            ink::selector_bytes!("transfer"),
                        ))
                        .push_arg(payment_info.patron)
                        .push_arg(payment_info.value),
                    )
                    .returns::<bool>();
                self.env().emit_event(TokenOutgoing {
                    id: id,
                    receiver: payment_info.patron,
                    amount: payment_info.value,
                });
                self.env().emit_event(AuditInfoUpdated {
                    id: Some(id),
                    payment_info: Some(self.audit_id_to_payment_info.get(id).unwrap()),
                    updated_by: Some(self.env().caller()),
                });
                self.audit_id_to_payment_info.insert(id, &payment_info);
                return true;
            }
            false
        }
    }
}

#[cfg(test)]
mod test_cases {
    use ink::primitives::AccountId;

    use super ::*;
    #[cfg(feature = "ink-experimental-engine")]
    use crate::digital_certificate::digital_certificate;
    fn random_account_id() -> AccountId {
        AccountId::from([0x42;32])
    }

    #[test]
    fn test_case_access_current_audit_id() {
        let accounts =
        ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.bob);
        let contract = escrow::Escrow::new(accounts.alice);
        assert_eq!(contract.get_current_audit_id(),0);
        println!("I'm here.");
    }
//     #[test]
//     fn test_case_make_new_audit() {
//         let accounts =
//         ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
//         ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.bob);
//         let mut contract = escrow::Escrow::new();
//         let value: openbrush::traits::Balance = 100;
//         let auditor = accounts.bob;
//         let arbiter_provider = accounts.charlie;
//         let deadline: u64 = 1688329909;

//         contract.create_new_payment(value, auditor, arbiter_provider, deadline);
//         assert_eq!(contract.get_paymentinfo(contract.get_current_audit_id()-1).unwrap().deadline, deadline);
//     }

//     #[test]
//     fn test_case_request_change_deadline() {
//         let accounts =
//         ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
//         ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.bob);
//         let mut contract = escrow::Escrow::new();
//         let value: openbrush::traits::Balance = 100;
//         let auditor = accounts.bob;
//         let arbiter_provider = accounts.charlie;
//         let deadline: u64 = 1688329909;
//         let haircut: openbrush::traits::Balance = 5;

//         contract.create_new_payment(value, auditor, arbiter_provider, deadline);
//         //line for checking if our payment info's audit id has been assigned.
//         assert_eq!(contract.get_paymentinfo(contract.get_current_audit_id()-1).unwrap().value, 100);
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
//         contract.request_additional_time(0, 1698329909, haircut);
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
//         assert_eq!(contract.approve_additional_time(0), true);
//         // println!(contract.get_payment_info(0).unwrap().deadline);
//         //throwing string error.
//         assert_eq!(contract.audit_id_to_payment_info.get(0).unwrap().deadline, 1698329909);
//         assert_eq!(contract.audit_id_to_payment_info.get(0).unwrap().value, value * (100-haircut)/100);
//     }

//     #[test]
//     fn test_case_arbiter_extends_deadline() {
//         let accounts =
//         ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
//         ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.bob);
//         let mut contract = escrow::Escrow::new();
//         let value: openbrush::traits::Balance = 100;
//         let auditor = accounts.bob;
//         let arbiter_provider = accounts.charlie;
//         let deadline: u64 = 1687329909;
//         let _haircut: openbrush::traits::Balance = 5;

//         contract.create_new_payment(value, auditor, arbiter_provider, deadline);

//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
//         let ipfs_hash= "blah-blah";
//         println!("printing my ipfs hash here {}", ipfs_hash);
//         assert_eq!(contract.mark_submitted(0, ipfs_hash.into()), true);
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
//         contract.assess_audit(0, false);
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
//         let _newtime:u64= 1688329909;
//         let _arbiters_haircut = 5;
//         contract.arbiters_extend_deadline(0, _newtime, _arbiters_haircut);
//         assert_eq!(contract.audit_id_to_payment_info.get(0).unwrap().deadline, _newtime);
//         println!("At the end of the testcase.");
//     }

//     #[test]
//     fn test_case_p1() {
//         //p1 is I->II.a->III.a
//         // so patron creates the auditID,
//         //the auditor will submit,
//         //patron accepts,

//         let accounts =
//         ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
//         ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.bob);
//         let mut contract = escrow::Escrow::new();
//         let value: openbrush::traits::Balance = 100;
//         let auditor = accounts.bob;
//         let arbiter_provider = accounts.charlie;
//         let deadline: u64 = 1687329909;
//         let _haircut: openbrush::traits::Balance = 5;
//         //I
//         contract.create_new_payment(value, auditor, arbiter_provider, deadline);
//         //II.a
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
//         let ipfs_hash = "yippiee";
//         assert_eq!(contract.mark_submitted(0, ipfs_hash.into()), true);
//         //III.a
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
//         contract.assess_audit(0, true);
//         assert_eq!(contract.audit_id_to_payment_info.get(0).unwrap().completed, true);
//     }

//     #[test]
//     fn test_case_p2 () {
//         //I->II.a->III.b.1
//         //p1 is I->II.a->III.a
//         // so patron creates the auditID,
//         //the auditor will submit,
//         //patron does not accept,
//         //arbiter_provider will extend the deadline with/without haircut off the value.

//         let accounts =
//         ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
//         ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.bob);
//         let mut contract = escrow::Escrow::new();
//         let value: openbrush::traits::Balance = 100;
//         let auditor = accounts.bob;
//         let arbiter_provider = accounts.charlie;
//         let deadline: u64 = 1687329909;
//         let _haircut: openbrush::traits::Balance = 5;
//         //I
//         contract.create_new_payment(value, auditor, arbiter_provider, deadline);
//         //II.a
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
//         let ipfs_hash = "yippiee";
//         assert_eq!(contract.mark_submitted(0, ipfs_hash.into()), true);
//         //III.b.1
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
//         contract.assess_audit(0, false);
//         assert_eq!(contract.audit_id_to_payment_info.get(0).unwrap().completed, false);
//         //arbiter provider increasing the deadline.
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
//         let new_deadline:u64 = 1688329909;
//         assert_eq!(contract.arbiters_extend_deadline(0,new_deadline , 0), true);
//         //II.a
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
//         let ipfs_hash = "new report link";
//         assert_eq!(contract.mark_submitted(0, ipfs_hash.into()), true);
//         //III.a
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
//         contract.assess_audit(0, true);
//         assert_eq!(contract.audit_id_to_payment_info.get(0).unwrap().completed, true);

//     }

//     #[test]
//     fn test_case_p3 () {
//         //I->II.a->III.b.1
//         //p1 is I->II.a->III.a
//         // so patron creates the auditID,
//         //the auditor will submit,
//         //patron does not accept,
//         //arbiter_provider will extend the deadline with/without haircut off the value.

//         let accounts =
//         ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
//         ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
//         ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.bob);
//         let mut contract = escrow::Escrow::new();
//         let value: openbrush::traits::Balance = 100;
//         let auditor = accounts.bob;
//         let arbiter_provider = accounts.charlie;
//         let deadline: u64 = 1687329909;
//         let _haircut: openbrush::traits::Balance = 5;
//         //I
//         contract.create_new_payment(value, auditor, arbiter_provider, deadline);

//     }
}