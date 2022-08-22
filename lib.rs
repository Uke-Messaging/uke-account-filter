#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

/// TODO prolly add a custom data struct or enum for blocked allowed i nthe same list, so it wouldbe a single mapping filters: Mapping<AccountId, Vec<Filter>>
/// and Filter would be something like a struct Filter { id, blockedorallowed }
/// For now, we can just use separate lists tho

#[ink::contract]
mod uke_account_filter {

    use ink_prelude::vec::Vec;
    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    use ink_storage::{traits::SpreadAllocate, Mapping};

    /// Emitted whenever a new user is registered.
    #[ink(event)]
    pub struct OptIn {
        #[ink(topic)]
        id: AccountId,
        #[ink(topic)]
        status: bool,
    }

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct UkeAccountFilter {
        /// Ensures it is a registered account id.
        opted_in: Mapping<AccountId, bool>,
        /// If true, it allows all messages.  if false, all messages are deemed as invalid.
        global_filter: Mapping<AccountId, bool>,
        /// Creates a mapping of accounts with privilege to message
        allowed_accounts: Mapping<AccountId, Vec<AccountId>>,
        /// Default contract address
        default_address: AccountId,
    }

    /// Errors that can occur upon calling this contract.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if the user is not opted in to be filtered
        NotOptedIn,
        /// Returned if caller is not owner while required to.
        CallerIsNotOwner,
    }

    /// Type alias for the contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl UkeAccountFilter {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.default_address = Default::default();
            })
        }

        /// Gets the optin status
        #[ink(message)]
        pub fn get_optin_status(&self, id: AccountId) -> bool {
            self.get_optin_status_or_default(id)
        }

        /// Changes optin status
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

        /// Toggles global filter for account
        #[ink(message)]
        pub fn change_global_filter(&mut self, id: AccountId, status: bool) -> Result<()> {
            if !self.is_caller_owner(id) {
                return Err(Error::CallerIsNotOwner);
            }
            self.global_filter.insert(&id, &status);
            Ok(())
        }

        /// Gets the global status status
        #[ink(message)]
        pub fn get_global_filter(&self, id: AccountId) -> bool {
            self.get_global_status_or_default(id)
        }

        /// Privileged access to contact the user
        #[ink(message)]
        pub fn add_to_allowed(&mut self, id: AccountId, id_to_add: AccountId) -> Result<()> {
            if !self.is_caller_owner(id) {
                return Err(Error::CallerIsNotOwner);
            }
            let mut vec = self.get_allowed_list_or_default(id);
            vec.push(id_to_add);
            self.allowed_accounts.insert(&id, &vec);
            Ok(())
        }

        /// Gets account whitelist
        #[ink(message)]
        pub fn get_allowed_accounts(&self, id: AccountId) -> Vec<AccountId> {
            self.get_allowed_list_or_default(id)
        }

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

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
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
                Ok(())
            );
        }

        #[ink::test]
        fn get_allowed_accounts_works() {
            let default_accounts = default_accounts();
            set_next_caller(default_accounts.alice);

            let mut contract = UkeAccountFilter::new();
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
