#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod uke_account_filter {

    use ink_prelude::vec::Vec;
    use ink_storage::{traits::SpreadAllocate, Mapping};

    /// Emitted whenever a new user changes their opt-in status.
    #[ink(event)]
    pub struct OptIn {
        #[ink(topic)]
        id: AccountId,
        #[ink(topic)]
        status: bool,
    }

    /// Uke Account Filters ink! Smart Contract.  
    /// Used for defining rules related to accounts that message using the uke protocol.
    ///
    /// # Description
    ///
    /// Users can define rules for whether they wish to be contacted or not, and who can contact them.
    /// They essentially can create whitelists to explicitly allow who is permitted to message that specific account,
    /// along with what data can be sent in the future.
    /// This measure prevents a common issue with phone numbers, email, and even other apps - spam.
    /// This contract ensures the rules are kept in place, and the user is safe from any malicious or unwanted messages.

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct UkeAccountFilter {
        /// Creates a mapping of whether an account is opted in or not.
        opted_in: Mapping<AccountId, bool>,
        /// If true, it allows all messages.  if false, all messages are deemed as invalid (except those in the whitelist).
        global_filter: Mapping<AccountId, bool>,
        /// Creates a mapping of accounts with privilege to message (whitelist).
        allowed_accounts: Mapping<AccountId, Vec<AccountId>>,
        /// Default contract address
        default_address: AccountId,
    }

    /// Errors that can occur upon calling this contract.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if the user is not opted in to interact with filters.
        NotOptedIn,
        /// Returned if caller is not owner while required to.
        CallerIsNotOwner,
    }

    /// Type alias for the contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl UkeAccountFilter {
        /// Creates a new account filter contract
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.default_address = Default::default();
            })
        }

        /// Gets the opt-in status of a selected account.
        #[ink(message)]
        pub fn get_optin_status(&self, id: AccountId) -> bool {
            self.get_optin_status_or_default(id)
        }

        /// Changes opt-in status of the selected account.
        #[ink(message)]
        pub fn change_optin_status(&mut self, status: bool, id: AccountId) -> Result<()> {
            if !self.is_caller_owner(id) {
                return Err(Error::CallerIsNotOwner);
            }
            // Update the status
            self.opted_in.insert(&id, &status);
            self.env().emit_event(OptIn { id, status });
            Ok(())
        }

        /// Changes global filter for the selected account.
        #[ink(message)]
        pub fn change_global_filter(&mut self, id: AccountId, status: bool) -> Result<()> {
            if !self.is_caller_owner(id) {
                return Err(Error::CallerIsNotOwner);
            } else if !self.get_optin_status_or_default(id) {
                return Err(Error::NotOptedIn);
            }

            self.global_filter.insert(&id, &status);
            Ok(())
        }

        /// Gets the global filter status of the selected account.
        #[ink(message)]
        pub fn get_global_filter(&self, id: AccountId) -> bool {
            self.get_global_status_or_default(id)
        }

        /// Adds a new user to the whitelist.
        #[ink(message)]
        pub fn add_to_allowed(&mut self, id: AccountId, id_to_add: AccountId) -> Result<()> {
            if !self.is_caller_owner(id) {
                return Err(Error::CallerIsNotOwner);
            } else if !self.get_optin_status_or_default(id) {
                return Err(Error::NotOptedIn);
            }

            let mut vec = self.get_allowed_list_or_default(id);
            vec.push(id_to_add);
            self.allowed_accounts.insert(&id, &vec);
            Ok(())
        }

        /// Gets account whitelist.
        #[ink(message)]
        pub fn get_allowed_accounts(&self, id: AccountId) -> Vec<AccountId> {
            self.get_allowed_list_or_default(id)
        }

        // Utility functions to ensure safe retrieval of various mappings.

        fn get_optin_status_or_default(&self, id: AccountId) -> bool {
            self.opted_in.get(&id).unwrap_or(false)
        }

        fn get_global_status_or_default(&self, id: AccountId) -> bool {
            self.global_filter.get(&id).unwrap_or(false)
        }

        fn is_caller_owner(&self, id: AccountId) -> bool {
            id == self.env().caller()
        }

        fn get_allowed_list_or_default(&self, id: AccountId) -> Vec<AccountId> {
            self.allowed_accounts.get(&id).unwrap_or(Vec::new())
        }
    }

    #[cfg(test)]
    mod tests {

        use super::*;
        use ink_lang as ink;

        fn set_next_caller(caller: AccountId) {
            ink_env::test::set_caller::<Environment>(caller);
        }

        fn default_accounts() -> ink_env::test::DefaultAccounts<ink_env::DefaultEnvironment> {
            ink_env::test::default_accounts::<Environment>()
        }

        #[ink::test]
        fn default_works() {
            let uke_account_filter = UkeAccountFilter::new();
            assert_eq!(uke_account_filter.default_address, Default::default());
        }

        #[ink::test]
        fn change_optin_works() {
            let default_accounts = default_accounts();
            set_next_caller(default_accounts.alice);

            let mut contract = UkeAccountFilter::new();

            assert_eq!(
                contract.add_to_allowed(default_accounts.alice, default_accounts.bob),
                Err(Error::NotOptedIn)
            );

            assert_eq!(
                contract.change_optin_status(true, default_accounts.alice),
                Ok(())
            );
            assert_eq!(
                contract.change_optin_status(true, default_accounts.bob),
                Err(Error::CallerIsNotOwner)
            );
        }

        #[ink::test]
        fn get_optin_works() {
            let default_accounts = default_accounts();
            set_next_caller(default_accounts.alice);

            let mut contract = UkeAccountFilter::new();
            contract
                .change_optin_status(true, default_accounts.alice)
                .unwrap();
            assert_eq!(contract.get_optin_status(default_accounts.alice), true);
        }

        #[ink::test]
        fn change_global_filter_works() {
            let default_accounts = default_accounts();
            set_next_caller(default_accounts.alice);

            let mut contract = UkeAccountFilter::new();

            assert_eq!(
                contract.change_global_filter(default_accounts.alice, true),
                Err(Error::NotOptedIn)
            );

            contract
                .change_optin_status(true, default_accounts.alice)
                .unwrap();

            assert_eq!(
                contract.change_global_filter(default_accounts.alice, true),
                Ok(())
            );

            assert_eq!(
                contract.change_global_filter(default_accounts.bob, true),
                Err(Error::CallerIsNotOwner)
            );
        }

        #[ink::test]
        fn get_global_filter_works() {
            let default_accounts = default_accounts();
            set_next_caller(default_accounts.alice);

            let mut contract = UkeAccountFilter::new();

            contract
                .change_optin_status(true, default_accounts.alice)
                .unwrap();

            contract
                .change_global_filter(default_accounts.alice, true)
                .unwrap();
            assert_eq!(contract.get_global_filter(default_accounts.alice), true);
        }

        #[ink::test]
        fn add_to_allowed_accounts_works() {
            let default_accounts = default_accounts();
            set_next_caller(default_accounts.alice);

            let mut contract = UkeAccountFilter::new();

            assert_eq!(
                contract.add_to_allowed(default_accounts.alice, default_accounts.bob),
                Err(Error::NotOptedIn)
            );

            contract
                .change_optin_status(true, default_accounts.alice)
                .unwrap();

            assert_eq!(
                contract.add_to_allowed(default_accounts.alice, default_accounts.bob),
                Ok(())
            );
        }

        #[ink::test]
        fn get_allowed_accounts_works() {
            let default_accounts = default_accounts();
            set_next_caller(default_accounts.alice);

            let mut contract = UkeAccountFilter::new();

            contract
                .change_optin_status(true, default_accounts.alice)
                .unwrap();

            contract
                .add_to_allowed(default_accounts.alice, default_accounts.bob)
                .unwrap();
            contract
                .add_to_allowed(default_accounts.alice, default_accounts.charlie)
                .unwrap();
            contract
                .add_to_allowed(default_accounts.alice, default_accounts.django)
                .unwrap();

            let allowed_list = contract.get_allowed_accounts(default_accounts.alice);

            assert_eq!(allowed_list.len(), 3);
            assert_eq!(allowed_list[0], default_accounts.bob);
        }
    }
}
