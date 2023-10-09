#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod voting {
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Arbiter {
        pub voter_address: AccountId,
        pub has_voted: bool,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    ///VoteInfo will store crucial information about the voting
    /// like the vector of arbiters, how many arbiters/voters are there, decided deadline, and haircut will update
    pub struct VoteInfo {
        pub audit_id: u32,
        pub arbiters: Vec<Arbiter>,
        pub is_active: bool,
        pub available_votes: u8,
        pub decided_deadline: Timestamp,
        pub decided_haircut: Balance,
        pub admin_hit_time: Timestamp,
    }
    pub type Result<T> = core::result::Result<T, Error>;

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum AuditArbitrationResult {
        NoDiscrepancies,
        MinorDiscrepancies,
        ModerateDiscrepancies,
        Reject,
    }

    #[ink(event)]
    pub struct PollCreated {
        id: u32,
        vote_info: VoteInfo,
    }

    #[ink(event)]
    pub struct ArbiterVoted {
        id: u32,
        voter: AccountId,
        vote_type: Option<AuditArbitrationResult>,
    }

    #[ink(event)]
    pub struct FinalVotePushed {
        id: u32,
        pusher: AccountId,
    }

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
        ValueTooLow,
        ValueTooHigh,
    }

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Voting {
        pub current_vote_id: u32,
        pub escrow_address: AccountId,
        pub stablecoin_address: AccountId,
        pub admin: AccountId,
        pub vote_id_to_info: Mapping<u32, VoteInfo>,
        pub vote_id_to_treasury: Mapping<u32, Balance>,
        pub haircut_for_minor_discreapancies: Balance,
        pub haircut_for_moderate_discrepancies: Balance,
        pub time_extension_for_minor_discrepancies: Timestamp,
        pub time_extension_for_moderate_discrepancies: Timestamp,
        pub arbiters_share: Balance,
    }

    impl Voting {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(
            _escrow_address: AccountId,
            _stablecoin_address: AccountId,
            _admin: AccountId,
        ) -> Self {
            let current_vote_id = u32::default();
            let vote_id_to_info = Mapping::default();
            let vote_id_to_treasury = Mapping::default();
            let escrow_address = _escrow_address;
            let stablecoin_address = _stablecoin_address;
            let admin = _admin;
            let haircut_for_minor_discreapancies = 5;
            let haircut_for_moderate_discrepancies = 15;
            let time_extension_for_minor_discrepancies = 604800000;
            let time_extension_for_moderate_discrepancies = 1296000000;
            let arbiters_share = 5;

            Self {
                current_vote_id,
                vote_id_to_info,
                vote_id_to_treasury,
                escrow_address,
                stablecoin_address,
                admin,
                haircut_for_minor_discreapancies,
                haircut_for_moderate_discrepancies,
                time_extension_for_minor_discrepancies,
                time_extension_for_moderate_discrepancies,
                arbiters_share,
            }
        }

        #[ink(message)]
        pub fn get_current_vote_id(&self) -> u32 {
            self.current_vote_id
        }

        #[ink(message)]
        pub fn know_your_escrow(&self) -> AccountId {
            self.escrow_address
        }

        #[ink(message)]
        pub fn know_your_admin(&self) -> AccountId {
            self.admin
        }

        #[ink(message)]
        pub fn get_poll_info(&self, _id: u32) -> Option<VoteInfo> {
            self.vote_id_to_info.get(&_id)
        }

        #[ink(message)]
        pub fn get_treasury_info(&self, _id: u32) -> Option<Balance> {
            self.vote_id_to_treasury.get(&_id)
        }

        ///create_new_poll can only be called by the admin of this contract, and will be called when patron rejects a submitted report
        /// the function takes the audit id of the audit under dispute and a list of arbiters who are going to vote on this proposal
        #[ink(message)]
        pub fn create_new_poll(
            &mut self,
            _audit_id: u32,
            _buffer_for_admin: Timestamp,
            _arbiters: Vec<Arbiter>,
        ) -> Result<()> {
            if self.env().caller() != self.admin {
                return Err(Error::UnAuthorisedCall);
            }
            let x = VoteInfo {
                audit_id: _audit_id,
                arbiters: _arbiters,
                is_active: true,
                available_votes: 0,
                decided_deadline: 0,
                decided_haircut: 0,
                admin_hit_time: _buffer_for_admin,
            };
            self.vote_id_to_info.insert(self.current_vote_id, &x);
            self.env().emit_event(PollCreated {
                id: self.current_vote_id,
                vote_info: x,
            });
            self.current_vote_id = self.current_vote_id + 1;
            return Ok(());
        }

        /// vote function is the main function of this contract, taking in vote_id and result as input by the arbiters,
        /// it first verifies that the voting is still active, and that the arbiter hasn't already voted.
        /// then it updates the state of this and the other contract according to stage.
        /// so if this is the final vote, it will directly call the other conract, similarly if the arbiter has selected reject,
        /// it will be a rejection without averaging out.
        /// But otherwise it will simply be compounded into decided_deadline and decided_haircut to be averaged out eventually.
        #[ink(message)]
        pub fn vote(&mut self, _vote_id: u32, _result: AuditArbitrationResult) -> Result<()> {
            let mut x = self.vote_id_to_info.get(_vote_id).unwrap();
            if !x.is_active {
                return Err(Error::ResultAlreadyPublished);
            }
            let mut index: usize = 0;
            for account in &x.arbiters {
                if account.voter_address == self.env().caller() {
                    break;
                }
                index = index + 1;
            }
            if index >= x.arbiters.len() {
                return Err(Error::UnAuthorisedCall);
            } else {
                if x.arbiters[index].has_voted {
                    return Err(Error::VotingFailed);
                } else {
                    //case when this is the last vote to be done... submit thing..
                    if x.available_votes + 1 == x.arbiters.len() as u8 {
                        match _result {
                            AuditArbitrationResult::NoDiscrepancies => {
                                if x.decided_deadline > 0 {
                                    x.decided_deadline =
                                        (x.decided_deadline) / (x.available_votes as Timestamp + 1);
                                    x.decided_haircut =
                                        (x.decided_haircut) / (x.available_votes as Balance + 1);

                                    let result_call = ink::env::call::build_call::<Environment>()
                                        .call(self.escrow_address)
                                        .gas_limit(0)
                                        .transferred_value(0)
                                        .exec_input(
                                            ink::env::call::ExecutionInput::new(
                                                ink::env::call::Selector::new(
                                                    ink::selector_bytes!(
                                                        "arbiters_extend_deadline"
                                                    ),
                                                ),
                                            )
                                            .push_arg(&x.audit_id)
                                            .push_arg(
                                                &x.decided_deadline + self.env().block_timestamp(),
                                            )
                                            .push_arg(&x.decided_haircut)
                                            .push_arg(self.arbiters_share)
                                            .push_arg(_vote_id),
                                        )
                                        .returns::<Result<()>>()
                                        .try_invoke();
                                    if matches!(result_call.unwrap().unwrap(), Result::Ok(())) {
                                        x.is_active = false;
                                        x.available_votes = x.available_votes + 1;
                                        x.arbiters[index].has_voted = true;
                                        self.vote_id_to_info.insert(_vote_id, &x);
                                        self.env().emit_event(ArbiterVoted {
                                            id: _vote_id,
                                            voter: self.env().caller(),
                                            vote_type: Some(_result),
                                        });
                                        self.env().emit_event(FinalVotePushed {
                                            id: _vote_id,
                                            pusher: self.env().caller(),
                                        });
                                        return Ok(());
                                    } else {
                                        return Err(Error::AssessmentFailed);
                                    }
                                } else {
                                    let result_call = ink::env::call::build_call::<Environment>()
                                        .call(self.escrow_address)
                                        .gas_limit(0)
                                        .transferred_value(0)
                                        .exec_input(
                                            ink::env::call::ExecutionInput::new(
                                                ink::env::call::Selector::new(
                                                    ink::selector_bytes!("assess_audit"),
                                                ),
                                            )
                                            .push_arg(&x.audit_id)
                                            .push_arg(true)
                                            .push_arg(_vote_id),
                                        )
                                        .returns::<Result<()>>()
                                        .try_invoke();
                                    if matches!(result_call.unwrap().unwrap(), Result::Ok(())) {
                                        x.available_votes = x.available_votes + 1;
                                        x.arbiters[index].has_voted = true;
                                        x.is_active = false;
                                        self.vote_id_to_info.insert(_vote_id, &x);
                                        self.env().emit_event(ArbiterVoted {
                                            id: _vote_id,
                                            voter: self.env().caller(),
                                            vote_type: Some(_result),
                                        });
                                        self.env().emit_event(FinalVotePushed {
                                            id: _vote_id,
                                            pusher: self.env().caller(),
                                        });
                                        return Ok(());
                                    } else {
                                        return Err(Error::AssessmentFailed);
                                    }
                                }
                            }
                            AuditArbitrationResult::MinorDiscrepancies => {
                                //add 7 days to the deadline extension.
                                x.decided_deadline = (x.decided_deadline
                                    + self.time_extension_for_minor_discrepancies)
                                    / (x.available_votes as Timestamp + 1);
                                x.decided_haircut = (x.decided_haircut
                                    + self.haircut_for_minor_discreapancies)
                                    / (x.available_votes as Balance + 1);
                                let result_call = ink::env::call::build_call::<Environment>()
                                    .call(self.escrow_address)
                                    .gas_limit(0)
                                    .transferred_value(0)
                                    .exec_input(
                                        ink::env::call::ExecutionInput::new(
                                            ink::env::call::Selector::new(ink::selector_bytes!(
                                                "arbiters_extend_deadline"
                                            )),
                                        )
                                        .push_arg(&x.audit_id)
                                        .push_arg(
                                            &x.decided_deadline + self.env().block_timestamp(),
                                        )
                                        .push_arg(&x.decided_haircut)
                                        .push_arg(self.arbiters_share)
                                        .push_arg(_vote_id),
                                    )
                                    .returns::<Result<()>>()
                                    .try_invoke();
                                if matches!(result_call.unwrap().unwrap(), Result::Ok(())) {
                                    x.available_votes = x.available_votes + 1;
                                    x.arbiters[index].has_voted = true;
                                    x.is_active = false;
                                    self.vote_id_to_info.insert(_vote_id, &x);
                                    self.env().emit_event(ArbiterVoted {
                                        id: _vote_id,
                                        voter: self.env().caller(),
                                        vote_type: Some(_result),
                                    });
                                    self.env().emit_event(FinalVotePushed {
                                        id: _vote_id,
                                        pusher: self.env().caller(),
                                    });
                                    return Ok(());
                                } else {
                                    return Err(Error::AssessmentFailed);
                                }
                            }
                            AuditArbitrationResult::ModerateDiscrepancies => {
                                //add 15 days to the deadline extension.
                                x.decided_deadline = (x.decided_deadline
                                    + self.time_extension_for_moderate_discrepancies)
                                    / (x.available_votes as Timestamp + 1);
                                x.decided_haircut = (x.decided_haircut
                                    + self.haircut_for_moderate_discrepancies)
                                    / (x.available_votes as Balance + 1);
                                let result_call = ink::env::call::build_call::<Environment>()
                                    .call(self.escrow_address)
                                    .gas_limit(0)
                                    .transferred_value(0)
                                    .exec_input(
                                        ink::env::call::ExecutionInput::new(
                                            ink::env::call::Selector::new(ink::selector_bytes!(
                                                "arbiters_extend_deadline"
                                            )),
                                        )
                                        .push_arg(&x.audit_id)
                                        .push_arg(
                                            &x.decided_deadline + self.env().block_timestamp(),
                                        )
                                        .push_arg(&x.decided_haircut)
                                        .push_arg(self.arbiters_share)
                                        .push_arg(_vote_id),
                                    )
                                    .returns::<Result<()>>()
                                    .try_invoke();
                                if matches!(result_call.unwrap().unwrap(), Result::Ok(())) {
                                    x.available_votes = x.available_votes + 1;
                                    x.arbiters[index].has_voted = true;
                                    x.is_active = false;
                                    self.vote_id_to_info.insert(_vote_id, &x);
                                    self.env().emit_event(ArbiterVoted {
                                        id: _vote_id,
                                        voter: self.env().caller(),
                                        vote_type: Some(_result),
                                    });
                                    self.env().emit_event(FinalVotePushed {
                                        id: _vote_id,
                                        pusher: self.env().caller(),
                                    });
                                    return Ok(());
                                } else {
                                    return Err(Error::AssessmentFailed);
                                }
                            }
                            AuditArbitrationResult::Reject => {
                                //call the function that rejects the audit report.
                                let result_call = ink::env::call::build_call::<Environment>()
                                    .call(self.escrow_address)
                                    .gas_limit(0)
                                    .transferred_value(0)
                                    .exec_input(
                                        ink::env::call::ExecutionInput::new(
                                            ink::env::call::Selector::new(ink::selector_bytes!(
                                                "assess_audit"
                                            )),
                                        )
                                        .push_arg(&x.audit_id)
                                        .push_arg(false)
                                        .push_arg(_vote_id),
                                    )
                                    .returns::<Result<()>>()
                                    .try_invoke();
                                if matches!(result_call.unwrap().unwrap(), Result::Ok(())) {
                                    x.available_votes = x.available_votes + 1;
                                    x.arbiters[index].has_voted = true;
                                    x.is_active = false;
                                    self.vote_id_to_info.insert(_vote_id, &x);
                                    self.env().emit_event(ArbiterVoted {
                                        id: _vote_id,
                                        voter: self.env().caller(),
                                        vote_type: Some(_result),
                                    });
                                    self.env().emit_event(FinalVotePushed {
                                        id: _vote_id,
                                        pusher: self.env().caller(),
                                    });
                                    return Ok(());
                                } else {
                                    return Err(Error::AssessmentFailed);
                                }
                            }
                        }
                    } else {
                        match _result {
                            AuditArbitrationResult::NoDiscrepancies => {
                                x.available_votes = x.available_votes + 1;
                                x.arbiters[index].has_voted = true;
                                self.vote_id_to_info.insert(_vote_id, &x);
                                self.env().emit_event(ArbiterVoted {
                                    id: _vote_id,
                                    voter: self.env().caller(),
                                    vote_type: Some(_result),
                                });
                                return Ok(());
                            }
                            AuditArbitrationResult::MinorDiscrepancies => {
                                x.available_votes = x.available_votes + 1;
                                x.arbiters[index].has_voted = true;
                                //add 7 days to the deadline extension.
                                x.decided_deadline = x.decided_deadline
                                    + self.time_extension_for_minor_discrepancies;
                                x.decided_haircut =
                                    x.decided_haircut + self.haircut_for_minor_discreapancies;
                                self.vote_id_to_info.insert(_vote_id, &x);
                                self.env().emit_event(ArbiterVoted {
                                    id: _vote_id,
                                    voter: self.env().caller(),
                                    vote_type: Some(_result),
                                });
                                return Ok(());
                            }
                            AuditArbitrationResult::ModerateDiscrepancies => {
                                x.available_votes = x.available_votes + 1;
                                x.arbiters[index].has_voted = true;
                                //add 15 days to the deadline extension.
                                x.decided_deadline = x.decided_deadline
                                    + self.time_extension_for_moderate_discrepancies;
                                x.decided_haircut =
                                    x.decided_haircut + self.haircut_for_moderate_discrepancies;
                                self.vote_id_to_info.insert(_vote_id, &x);
                                self.env().emit_event(ArbiterVoted {
                                    id: _vote_id,
                                    voter: self.env().caller(),
                                    vote_type: Some(_result),
                                });
                                return Ok(());
                            }
                            AuditArbitrationResult::Reject => {
                                let result_call = ink::env::call::build_call::<Environment>()
                                    .call(self.escrow_address)
                                    .gas_limit(0)
                                    .transferred_value(0)
                                    .exec_input(
                                        ink::env::call::ExecutionInput::new(
                                            ink::env::call::Selector::new(ink::selector_bytes!(
                                                "assess_audit"
                                            )),
                                        )
                                        .push_arg(&x.audit_id)
                                        .push_arg(false)
                                        .push_arg(_vote_id),
                                    )
                                    .returns::<Result<()>>()
                                    .try_invoke();
                                if matches!(result_call.unwrap().unwrap(), Result::Ok(())) {
                                    x.available_votes = x.available_votes + 1;
                                    x.arbiters[index].has_voted = true;
                                    x.is_active = false;
                                    self.vote_id_to_info.insert(_vote_id, &x);
                                    self.env().emit_event(ArbiterVoted {
                                        id: _vote_id,
                                        voter: self.env().caller(),
                                        vote_type: Some(_result),
                                    });
                                    self.env().emit_event(FinalVotePushed {
                                        id: _vote_id,
                                        pusher: self.env().caller(),
                                    });
                                    return Ok(());
                                } else {
                                    return Err(Error::AssessmentFailed);
                                }
                            }
                        }
                    }
                }
            }
        }

        #[ink(message)]
        pub fn add_to_treasury(&mut self, _vote_id: u32, _funds: Balance) -> Result<()> {
            if self.env().caller() != self.escrow_address {
                return Err(Error::UnAuthorisedCall);
            }
            let bal = _funds;
            self.vote_id_to_treasury.insert(_vote_id, &bal);
            Ok(())
        }

        #[ink(message)]
        pub fn release_treasury_funds(&mut self, _vote_id: u32) -> Result<()> {
            if self.env().caller() != self.admin {
                return Err(Error::UnAuthorisedCall);
            }
            let available_balance = self.vote_id_to_treasury.get(_vote_id).unwrap();
            if available_balance == 0 {
                return Err(Error::TreasuryEmpty);
            }
            let vote_info = self.vote_id_to_info.get(_vote_id).unwrap();
            let total_voters = vote_info.available_votes;
            let per_voter_share = available_balance / (total_voters as Balance);
            for i in 0..(vote_info.arbiters.len() - 1) {
                let arbiter_info = vote_info.arbiters.get(i).unwrap();
                if arbiter_info.has_voted {
                    let _red = ink::env::call::build_call::<Environment>()
                        .call(self.stablecoin_address)
                        .gas_limit(0)
                        .transferred_value(0)
                        .exec_input(
                            ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                                ink::selector_bytes!("transfer"),
                            ))
                            .push_arg(arbiter_info.voter_address)
                            .push_arg(per_voter_share), // .push_arg(&[0x10u8; 32]),
                        )
                        .returns::<Result<()>>()
                        .try_invoke();
                }
            }
            Ok(())
        }
        ///In case when not all arbiters have voted on a particular proposal, the admin has the liberty of forcing the vote by submitting the
        /// current decision, accordingly it will either approve the auditor or extend their deadline.
        #[ink(message)]
        pub fn force_vote(&mut self, _vote_id: u32) -> Result<()> {
            if self.env().caller() != self.admin {
                return Err(Error::UnAuthorisedCall);
            }
            if self.vote_id_to_info.get(_vote_id).unwrap().admin_hit_time
                > self.env().block_timestamp()
            {
                return Err(Error::RightsNotActivatedYet);
            }
            let mut x = self.vote_id_to_info.get(_vote_id).unwrap();

            if !x.is_active {
                return Err(Error::ResultAlreadyPublished);
            }
            if x.decided_deadline > 0 {
                let result_call = ink::env::call::build_call::<Environment>()
                    .call(self.escrow_address)
                    .gas_limit(0)
                    .transferred_value(0)
                    .exec_input(
                        ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                            ink::selector_bytes!("arbiters_extend_deadline"),
                        ))
                        .push_arg(&x.audit_id)
                        .push_arg(&x.decided_deadline + self.env().block_timestamp())
                        .push_arg(&x.decided_haircut)
                        .push_arg(self.arbiters_share)
                        .push_arg(_vote_id),
                    )
                    .returns::<Result<()>>()
                    .try_invoke();
                if matches!(result_call.unwrap().unwrap(), Result::Ok(())) {
                    x.is_active = false;
                    x.decided_deadline = (x.decided_deadline) / (x.available_votes as Timestamp);
                    x.decided_haircut = (x.decided_haircut) / (x.available_votes as Balance);
                    self.vote_id_to_info.insert(_vote_id, &x);
                    self.env().emit_event(FinalVotePushed {
                        id: _vote_id,
                        pusher: self.env().caller(),
                    });
                    return Ok(());
                } else {
                    return Err(Error::AssessmentFailed);
                }
            } else if x.decided_deadline == 0 {
                let result_call = ink::env::call::build_call::<Environment>()
                    .call(self.escrow_address)
                    .gas_limit(0)
                    .transferred_value(0)
                    .exec_input(
                        ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                            ink::selector_bytes!("assess_audit"),
                        ))
                        .push_arg(&x.audit_id)
                        .push_arg(true)
                        .push_arg(_vote_id),
                    )
                    .returns::<Result<()>>()
                    .try_invoke();
                if matches!(result_call.unwrap().unwrap(), Result::Ok(())) {
                    x.is_active = false;
                    self.vote_id_to_info.insert(_vote_id, &x);
                    return Ok(());
                } else {
                    return Err(Error::AssessmentFailed);
                }
            }
            return Err(Error::UnAuthorisedCall);
        }

        #[ink(message)]
        pub fn flush_out_tokens(
            &mut self,
            _token_address: AccountId,
            _value: Balance,
        ) -> Result<()> {
            let _result_call = ink::env::call::build_call::<Environment>()
                .call(_token_address)
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ink::env::call::ExecutionInput::new(ink::env::call::Selector::new(
                        ink::selector_bytes!("transfer"),
                    ))
                    .push_arg(&self.admin)
                    .push_arg(_value),
                )
                .returns::<Result<()>>()
                .try_invoke();
            if matches!(_result_call.unwrap().unwrap(), Result::Ok(())) {
                return Ok(());
            } else {
                return Err(Error::TransferFailed);
            }
        }

        #[ink(message)]
        pub fn change_haircut_for_discrepancies(
            &mut self,
            change_minor: bool,
            new_haircut: Balance,
        ) -> Result<()> {
            if self.env().caller() != self.admin {
                return Err(Error::UnAuthorisedCall);
            }
            if new_haircut > 90 {
                return Err(Error::ValueTooHigh);
            }
            if change_minor {
                self.haircut_for_minor_discreapancies = new_haircut;
            } else {
                self.haircut_for_moderate_discrepancies = new_haircut;
            }
            return Ok(());
        }

        #[ink(message)]
        pub fn change_time_extension_for_discrepancies(
            &mut self,
            change_minor: bool,
            new_extension: Timestamp,
        ) -> Result<()> {
            if self.env().caller() != self.admin {
                return Err(Error::UnAuthorisedCall);
            }
            if new_extension < 86400000 {
                return Err(Error::ValueTooLow);
            }
            if change_minor {
                self.time_extension_for_minor_discrepancies = new_extension;
            } else {
                self.time_extension_for_moderate_discrepancies = new_extension;
            }
            return Ok(());
        }

        #[ink(message)]
        pub fn change_arbiters_share(&mut self, new_share: Balance) -> Result<()> {
            if self.env().caller() != self.admin {
                return Err(Error::UnAuthorisedCall);
            }
            self.arbiters_share = new_share;
            Ok(())
        }
    }
}
