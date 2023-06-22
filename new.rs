#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod escrow {
    use ink::{storage::Mapping};
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
        pub audit_id_to_payment_info: Mapping<u32, PaymentInfo>,
        // for the transfers of funds to the arbiters, we will simply send the money to arbiterprovider in the withdrawal, and they are supposed to deal with the rest.
        pub audit_id_to_time_increase_request: ink::storage::Mapping<u32, IncreaseRequest>,
        pub audit_id_to_ipfs_hash: ink::storage::Mapping<u32, String>,
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
            println!("Entered the function approve_additional_time");
            if self.get_payment_info(&id).unwrap().patron == self.env().caller() {
                let haircut = self
                    .query_timeincreaserequest(id)
                    .unwrap()
                    .haircut_percentage;
                if haircut < 100 {
                    println!("Entered the haircut mark");
                    let new_deadline = self.query_timeincreaserequest(id).unwrap().newdeadline;


                    let mut payment_info = self.audit_id_to_payment_info.get(id).unwrap();
            // Update the value in storage
                payment_info.value = payment_info.value*(100 - haircut) / 100;
            // Update the deadline in storage
                payment_info.deadline = new_deadline;
            // Set the updated payment_info back to storage
                self.audit_id_to_payment_info.insert(id, &payment_info);
                    //if *= doesn't work, use "self.get_payment_info(&id).unwrap().value *"

                    // self.audit_id_to_payment_info.get(id).unwrap().value *= (100 - haircut) / 100;
                    // println!("Entered the line just before.");
                    // self.audit_id_to_payment_info.get(id).unwrap().deadline = new_deadline;
                    self.env().emit_event(AuditInfoUpdated {
                        id: Some(id),
                        payment_info: Some(self.audit_id_to_payment_info.get(id).unwrap()),
                        updated_by: Some(self.get_payment_info(&id).unwrap().patron),
                    });
                    println!("we can be sure that approve_extended deadline will return true");
                    return true;
                }
                println!(" Function is returning the internal false");
                return false;
            }
            println!(" Function is returning the external false");
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
                        let mut payment_info = self.audit_id_to_payment_info.get(_id).unwrap();
                        payment_info.completed = true;
                        self.audit_id_to_payment_info.insert(_id, &payment_info);

                    // to_do
                    //transferring tokens 98% to the auditor,
                    //transferring tokens 2% to the arbiter_provider for being there.
                    } else {
                        let mut payment_info = self.audit_id_to_payment_info.get(_id).unwrap();
                        payment_info.completed = true;
                        self.audit_id_to_payment_info.insert(_id, &payment_info);
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

//to_do 
// in every function where haircut is mentioned, return the rest of the amount to the patron. 
//complete this task after integrating psp22/erc20
        #[ink(message)]
        pub fn arbiters_extend_deadline(&mut self, _id: u32, new_deadline: u64, haircut: Balance) -> bool {
            //checking for the haircut to be lesser than 10% and new deadline to be at least more than 1 day.
            if haircut < 10 && new_deadline > self.env().block_timestamp() + 86400 {
                println!("At least the code got into the if case of arbiters_extend_deadline");

                let mut payment_info = self.audit_id_to_payment_info.get(_id).unwrap();
                // Update the value in storage
                payment_info.value = payment_info.value*(100 - haircut) / 100;
                // Update the deadline in storage
                payment_info.deadline = new_deadline;
                // Set the updated payment_info back to storage
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
        let contract = escrow::Escrow::new();
        assert_eq!(contract.get_current_audit_id(),0);
        println!("I'm here.");
    }
    #[test]
    fn test_case_make_new_audit() {
        let accounts = 
        ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.bob);
        let mut contract = escrow::Escrow::new();
        let value: openbrush::traits::Balance = 100;
        let auditor = accounts.bob;
        let arbiter_provider = accounts.charlie;
        let deadline: u64 = 1688329909;

        contract.create_new_payment(value, auditor, arbiter_provider, deadline);
        assert_eq!(contract.get_paymentinfo(contract.get_current_audit_id()-1).unwrap().deadline, deadline);
    }

    #[test]
    fn test_case_request_change_deadline() {
        let accounts = 
        ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.bob);
        let mut contract = escrow::Escrow::new();
        let value: openbrush::traits::Balance = 100;
        let auditor = accounts.bob;
        let arbiter_provider = accounts.charlie;
        let deadline: u64 = 1688329909;
        let haircut: openbrush::traits::Balance = 5;

        contract.create_new_payment(value, auditor, arbiter_provider, deadline);
        //line for checking if our payment info's audit id has been assigned.
        assert_eq!(contract.get_paymentinfo(contract.get_current_audit_id()-1).unwrap().value, 100);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        contract.request_additional_time(0, 1698329909, haircut);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        assert_eq!(contract.approve_additional_time(0), true);
        // println!(contract.get_payment_info(0).unwrap().deadline);
        //throwing string error.
        assert_eq!(contract.audit_id_to_payment_info.get(0).unwrap().deadline, 1698329909);
        assert_eq!(contract.audit_id_to_payment_info.get(0).unwrap().value, value * (100-haircut)/100);
    }

    #[test]
    fn test_case_arbiter_extends_deadline() {
        let accounts = 
        ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.bob);
        let mut contract = escrow::Escrow::new();
        let value: openbrush::traits::Balance = 100;
        let auditor = accounts.bob;
        let arbiter_provider = accounts.charlie;
        let deadline: u64 = 1687329909;
        let _haircut: openbrush::traits::Balance = 5;

        contract.create_new_payment(value, auditor, arbiter_provider, deadline);

        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        let ipfs_hash= "blah-blah";
        println!("printing my ipfs hash here {}", ipfs_hash);
        assert_eq!(contract.mark_completed(0, ipfs_hash.into()), true);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        contract.assess_audit(0, false);
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
        let _newtime:u64= 1688329909;
        let _arbiters_haircut = 5;
        contract.arbiters_extend_deadline(0, _newtime, _arbiters_haircut);
        assert_eq!(contract.audit_id_to_payment_info.get(0).unwrap().deadline, _newtime);
        println!("At the end of the testcase.");
    }


    #[test]
    fn test_case_p1() {
        //p1 is I->II.a->III.a 
        // so patron creates the auditID, 
        //the auditor will submit,
        //patron accepts, 

        let accounts = 
        ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.bob);
        let mut contract = escrow::Escrow::new();
        let value: openbrush::traits::Balance = 100;
        let auditor = accounts.bob;
        let arbiter_provider = accounts.charlie;
        let deadline: u64 = 1687329909;
        let _haircut: openbrush::traits::Balance = 5;
        //I
        contract.create_new_payment(value, auditor, arbiter_provider, deadline);
        //II.a
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        let ipfs_hash = "yippiee";
        assert_eq!(contract.mark_completed(0, ipfs_hash.into()), true);
        //III.a
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        contract.assess_audit(0, true);
        assert_eq!(contract.audit_id_to_payment_info.get(0).unwrap().completed, true);
    }

    #[test]
    fn test_case_p2 () {
        //I->II.a->III.b.1
        //p1 is I->II.a->III.a 
        // so patron creates the auditID, 
        //the auditor will submit,
        //patron does not accept, 
        //arbiter_provider will extend the deadline with/without haircut off the value.
        
        let accounts = 
        ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.bob);
        let mut contract = escrow::Escrow::new();
        let value: openbrush::traits::Balance = 100;
        let auditor = accounts.bob;
        let arbiter_provider = accounts.charlie;
        let deadline: u64 = 1687329909;
        let _haircut: openbrush::traits::Balance = 5;
        //I
        contract.create_new_payment(value, auditor, arbiter_provider, deadline);
        //II.a
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        let ipfs_hash = "yippiee";
        assert_eq!(contract.mark_completed(0, ipfs_hash.into()), true);
        //III.b.1
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        contract.assess_audit(0, false);
        assert_eq!(contract.audit_id_to_payment_info.get(0).unwrap().completed, false);
        //arbiter provider increasing the deadline.
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
        let new_deadline:u64 = 1688329909;
        assert_eq!(contract.arbiters_extend_deadline(0,new_deadline , 0), true);
        //II.a
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        let ipfs_hash = "new report link";
        assert_eq!(contract.mark_completed(0, ipfs_hash.into()), true);
        //III.a
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        contract.assess_audit(0, true);
        assert_eq!(contract.audit_id_to_payment_info.get(0).unwrap().completed, true);

    }

    #[test]
    fn test_case_p3 () {
        //I->II.a->III.b.1
        //p1 is I->II.a->III.a 
        // so patron creates the auditID, 
        //the auditor will submit,
        //patron does not accept, 
        //arbiter_provider will extend the deadline with/without haircut off the value.
        
        let accounts = 
        ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        ink::env::test::set_callee::<ink::env::DefaultEnvironment>(accounts.bob);
        let mut contract = escrow::Escrow::new();
        let value: openbrush::traits::Balance = 100;
        let auditor = accounts.bob;
        let arbiter_provider = accounts.charlie;
        let deadline: u64 = 1687329909;
        let _haircut: openbrush::traits::Balance = 5;
        //I
        contract.create_new_payment(value, auditor, arbiter_provider, deadline);
        
    }
}
