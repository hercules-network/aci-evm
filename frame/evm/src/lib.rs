#![cfg_attr(not(feature = "std"), no_std)]

pub mod runner;

pub use crate::runner::Runner;
pub use evm::{ExitError, ExitFatal, ExitReason, ExitRevert, ExitSucceed};
pub use ap_evm::{
    Account, CallInfo, CreateInfo, ExecutionInfo, LinearCostPrecompile, Log, Precompile,
    PrecompileSet, Vicinity,
};

#[cfg(feature = "std")]
use codec::{Decode, Encode};
use evm::Config as EvmConfig;
use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_support::traits::{Currency, ExistenceRequirement, Get, WithdrawReasons};
use frame_support::weights::{Pays, PostDispatchInfo, Weight};
use frame_support::{decl_error, decl_event, decl_module, decl_storage};
use frame_system::RawOrigin;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::{Hasher, H160, H256, U256};
use sp_runtime::{
    traits::{BadOrigin, UniqueSaturatedInto},
    AccountId32,
};
use sp_std::vec::Vec;

pub type BalanceOf<T> =
<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub trait FeeCalculator {
    fn min_gas_price() -> U256;
}

impl FeeCalculator for () {
    fn min_gas_price() -> U256 {
        U256::zero()
    }
}

pub trait EnsureAddressOrigin<OuterOrigin> {
    type Success;

    fn ensure_address_origin(
        address: &H160,
        origin: OuterOrigin,
    ) -> Result<Self::Success, BadOrigin> {
        Self::try_address_origin(address, origin).map_err(|_| BadOrigin)
    }

    fn try_address_origin(
        address: &H160,
        origin: OuterOrigin,
    ) -> Result<Self::Success, OuterOrigin>;
}

pub struct EnsureAddressSame;

impl<OuterOrigin> EnsureAddressOrigin<OuterOrigin> for EnsureAddressSame
    where
        OuterOrigin: Into<Result<RawOrigin<H160>, OuterOrigin>> + From<RawOrigin<H160>>,
{
    type Success = H160;

    fn try_address_origin(address: &H160, origin: OuterOrigin) -> Result<H160, OuterOrigin> {
        origin.into().and_then(|o| match o {
            RawOrigin::Signed(who) if &who == address => Ok(who),
            r => Err(OuterOrigin::from(r)),
        })
    }
}

pub struct EnsureAddressRoot<AccountId>(sp_std::marker::PhantomData<AccountId>);

impl<OuterOrigin, AccountId> EnsureAddressOrigin<OuterOrigin> for EnsureAddressRoot<AccountId>
    where
        OuterOrigin: Into<Result<RawOrigin<AccountId>, OuterOrigin>> + From<RawOrigin<AccountId>>,
{
    type Success = ();

    fn try_address_origin(_address: &H160, origin: OuterOrigin) -> Result<(), OuterOrigin> {
        origin.into().and_then(|o| match o {
            RawOrigin::Root => Ok(()),
            r => Err(OuterOrigin::from(r)),
        })
    }
}

pub struct EnsureAddressNever<AccountId>(sp_std::marker::PhantomData<AccountId>);

impl<OuterOrigin, AccountId> EnsureAddressOrigin<OuterOrigin> for EnsureAddressNever<AccountId> {
    type Success = AccountId;

    fn try_address_origin(_address: &H160, origin: OuterOrigin) -> Result<AccountId, OuterOrigin> {
        Err(origin)
    }
}

pub struct EnsureAddressTruncated;

impl<OuterOrigin> EnsureAddressOrigin<OuterOrigin> for EnsureAddressTruncated
    where
        OuterOrigin: Into<Result<RawOrigin<AccountId32>, OuterOrigin>> + From<RawOrigin<AccountId32>>,
{
    type Success = AccountId32;

    fn try_address_origin(address: &H160, origin: OuterOrigin) -> Result<AccountId32, OuterOrigin> {
        origin.into().and_then(|o| match o {
            RawOrigin::Signed(who) if AsRef::<[u8; 32]>::as_ref(&who)[0..20] == address[0..20] => {
                Ok(who)
            }
            r => Err(OuterOrigin::from(r)),
        })
    }
}

pub trait AddressMapping<A> {
    fn into_account_id(address: H160) -> A;
}

/// Identity address mapping.
pub struct IdentityAddressMapping;

impl AddressMapping<H160> for IdentityAddressMapping {
    fn into_account_id(address: H160) -> H160 {
        address
    }
}

pub struct HashedAddressMapping<H>(sp_std::marker::PhantomData<H>);

impl<H: Hasher<Out = H256>> AddressMapping<AccountId32> for HashedAddressMapping<H> {
    fn into_account_id(address: H160) -> AccountId32 {
        let mut data = [0u8; 24];
        data[0..4].copy_from_slice(b"evm:");
        data[4..24].copy_from_slice(&address[..]);
        let hash = H::hash(&data);

        AccountId32::from(Into::<[u8; 32]>::into(hash))
    }
}

pub trait GasWeightMapping {
    fn gas_to_weight(gas: u64) -> Weight;
    fn weight_to_gas(weight: Weight) -> u64;
}

impl GasWeightMapping for () {
    fn gas_to_weight(gas: u64) -> Weight {
        gas as Weight
    }
    fn weight_to_gas(weight: Weight) -> u64 {
        weight as u64
    }
}

static ISTANBUL_CONFIG: EvmConfig = EvmConfig::istanbul();

pub trait Config: frame_system::Config + pallet_timestamp::Config {
    type FeeCalculator: FeeCalculator;

    type GasWeightMapping: GasWeightMapping;

    type CallOrigin: EnsureAddressOrigin<Self::Origin>;

    type WithdrawOrigin: EnsureAddressOrigin<Self::Origin, Success = Self::AccountId>;

    type AddressMapping: AddressMapping<Self::AccountId>;

    type Currency: Currency<Self::AccountId>;

    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    type Precompiles: PrecompileSet;

    type ChainId: Get<u64>;

    type Runner: Runner<Self>;

    fn config() -> &'static EvmConfig {
        &ISTANBUL_CONFIG
    }
}

#[cfg(feature = "std")]
#[derive(Clone, Eq, PartialEq, Encode, Decode, Debug, Serialize, Deserialize)]
pub struct GenesisAccount {
    pub nonce: U256,
    pub balance: U256,
    pub storage: std::collections::BTreeMap<H256, H256>,
    pub code: Vec<u8>,
}

decl_storage! {
    trait Store for Module<T: Config> as EVM {
        AccountCodes get(fn account_codes): map hasher(blake2_128_concat) H160 => Vec<u8>;
        AccountStorages get(fn account_storages):
            double_map hasher(blake2_128_concat) H160, hasher(blake2_128_concat) H256 => H256;
    }

    add_extra_genesis {
        config(accounts): std::collections::BTreeMap<H160, GenesisAccount>;
        build(|config: &GenesisConfig| {
            for (address, account) in &config.accounts {
                let account_id = T::AddressMapping::into_account_id(*address);
                // `u128::max_value()`.
                for _ in 0..account.nonce.low_u128() {
                    frame_system::Module::<T>::inc_account_nonce(&account_id);
                }
                T::Currency::deposit_creating(
                    &account_id,
                    account.balance.low_u128().unique_saturated_into(),
                );
                AccountCodes::insert(address, &account.code);
                for (index, value) in &account.storage {
                    AccountStorages::insert(address, index, value);
                }
            }
        });
    }
}

decl_event! {
    pub enum Event<T> where
        <T as frame_system::Config>::AccountId,
    {
        Log(Log),
        Created(H160),
        CreatedFailed(H160),
        Executed(H160),
        ExecutedFailed(H160),
        BalanceDeposit(AccountId, H160, U256),
        BalanceWithdraw(AccountId, H160, U256),
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        BalanceLow,
        FeeOverflow,
        PaymentOverflow,
        WithdrawFailed,
        GasPriceTooLow,
        InvalidNonce,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        #[weight = 0]
        fn withdraw(origin, address: H160, value: BalanceOf<T>) {
            let destination = T::WithdrawOrigin::ensure_address_origin(&address, origin)?;
            let address_account_id = T::AddressMapping::into_account_id(address);
            T::Currency::transfer(
                &address_account_id,
                &destination,
                value,
                ExistenceRequirement::AllowDeath
            )?;
        }

        #[weight = T::GasWeightMapping::gas_to_weight(*gas_limit)]
        fn call(
            origin,
            source: H160,
            target: H160,
            input: Vec<u8>,
            value: U256,
            gas_limit: u64,
            gas_price: U256,
            nonce: Option<U256>,
        ) -> DispatchResultWithPostInfo {
            T::CallOrigin::ensure_address_origin(&source, origin)?;
            let info = T::Runner::call(
                source,
                target,
                input,
                value,
                gas_limit,
                Some(gas_price),
                nonce,
                T::config(),
            )?;
            match info.exit_reason {
                ExitReason::Succeed(_) => {
                    Module::<T>::deposit_event(Event::<T>::Executed(target));
                },
                _ => {
                    Module::<T>::deposit_event(Event::<T>::ExecutedFailed(target));
                },
            };
            Ok(PostDispatchInfo {
                actual_weight: Some(T::GasWeightMapping::gas_to_weight(info.used_gas.unique_saturated_into())),
                pays_fee: Pays::No,
            })
        }

        #[weight = T::GasWeightMapping::gas_to_weight(*gas_limit)]
        fn create(
            origin,
            source: H160,
            init: Vec<u8>,
            value: U256,
            gas_limit: u64,
            gas_price: U256,
            nonce: Option<U256>,
        ) -> DispatchResultWithPostInfo {
            T::CallOrigin::ensure_address_origin(&source, origin)?;
            let info = T::Runner::create(
                source,
                init,
                value,
                gas_limit,
                Some(gas_price),
                nonce,
                T::config(),
            )?;
            match info {
                CreateInfo {
                    exit_reason: ExitReason::Succeed(_),
                    value: create_address,
                    ..
                } => {
                    Module::<T>::deposit_event(Event::<T>::Created(create_address));
                },
                CreateInfo {
                    exit_reason: _,
                    value: create_address,
                    ..
                } => {
                    Module::<T>::deposit_event(Event::<T>::CreatedFailed(create_address));
                },
            }
            Ok(PostDispatchInfo {
                actual_weight: Some(T::GasWeightMapping::gas_to_weight(info.used_gas.unique_saturated_into())),
                pays_fee: Pays::No,
            })
        }

        #[weight = T::GasWeightMapping::gas_to_weight(*gas_limit)]
        fn create2(
            origin,
            source: H160,
            init: Vec<u8>,
            salt: H256,
            value: U256,
            gas_limit: u64,
            gas_price: U256,
            nonce: Option<U256>,
        ) -> DispatchResultWithPostInfo {
            T::CallOrigin::ensure_address_origin(&source, origin)?;
            let info = T::Runner::create2(
                source,
                init,
                salt,
                value,
                gas_limit,
                Some(gas_price),
                nonce,
                T::config(),
            )?;
            match info {
                CreateInfo {
                    exit_reason: ExitReason::Succeed(_),
                    value: create_address,
                    ..
                } => {
                    Module::<T>::deposit_event(Event::<T>::Created(create_address));
                },
                CreateInfo {
                    exit_reason: _,
                    value: create_address,
                    ..
                } => {
                    Module::<T>::deposit_event(Event::<T>::CreatedFailed(create_address));
                },
            }
            Ok(PostDispatchInfo {
                actual_weight: Some(T::GasWeightMapping::gas_to_weight(info.used_gas.unique_saturated_into())),
                pays_fee: Pays::No,
            })
        }
    }
}

impl<T: Config> Module<T> {
    pub fn is_account_empty(address: &H160) -> bool {
        let account = Self::account_basic(address);
        let code_len = AccountCodes::decode_len(address).unwrap_or(0);
        account.nonce == U256::zero() && account.balance == U256::zero() && code_len == 0
    }

    pub fn remove_account_if_empty(address: &H160) {
        if Self::is_account_empty(address) {
            Self::remove_account(address);
        }
    }

    pub fn remove_account(address: &H160) {
        AccountCodes::remove(address);
        AccountStorages::remove_prefix(address);
    }

    pub fn account_basic(address: &H160) -> Account {
        let account_id = T::AddressMapping::into_account_id(*address);
        let nonce = frame_system::Module::<T>::account_nonce(&account_id);
        let balance = T::Currency::free_balance(&account_id);
        Account {
            nonce: U256::from(UniqueSaturatedInto::<u128>::unique_saturated_into(nonce)),
            balance: U256::from(UniqueSaturatedInto::<u128>::unique_saturated_into(balance)),
        }
    }

    pub fn withdraw_fee(address: &H160, value: U256) -> Result<(), Error<T>> {
        let account_id = T::AddressMapping::into_account_id(*address);
        drop(
            T::Currency::withdraw(
                &account_id,
                value.low_u128().unique_saturated_into(),
                WithdrawReasons::FEE,
                ExistenceRequirement::AllowDeath,
            )
                .map_err(|_| Error::<T>::BalanceLow)?,
        );

        Ok(())
    }

    pub fn deposit_fee(address: &H160, value: U256) {
        let account_id = T::AddressMapping::into_account_id(*address);
        drop(T::Currency::deposit_creating(
            &account_id,
            value.low_u128().unique_saturated_into(),
        ));
    }
}