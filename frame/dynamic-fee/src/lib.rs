#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_event, decl_module, decl_storage, traits::Get};
use frame_system::ensure_none;
use sp_core::U256;
#[cfg(feature = "std")]
use sp_inherents::ProvideInherentData;
use sp_inherents::{InherentData, InherentIdentifier, IsFatalError, ProvideInherent};
use sp_runtime::RuntimeDebug;
use sp_std::{
    cmp::{max, min},
    result,
};

pub trait Config: frame_system::Config {
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;
    type MinGasPriceBoundDivisor: Get<U256>;
}

decl_storage! {
    trait Store for Module<T: Config> as DynamicFee {
        MinGasPrice get(fn min_gas_price) config(): U256;
        TargetMinGasPrice: Option<U256>;
    }
}

decl_event!(
    pub enum Event {
        TargetMinGasPriceSet(U256),
    }
);

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        fn deposit_event() = default;
        fn on_finalize(n: T::BlockNumber) {
            if let Some(target) = TargetMinGasPrice::get() {
                let bound = MinGasPrice::get() / T::MinGasPriceBoundDivisor::get() + U256::one();
                let upper_limit = MinGasPrice::get().saturating_add(bound);
                let lower_limit = MinGasPrice::get().saturating_sub(bound);
                MinGasPrice::set(min(upper_limit, max(lower_limit, target)));
            }
            TargetMinGasPrice::kill();
        }

        #[weight = 0]
        fn note_min_gas_price_target(
            origin,
            target: U256,
        ) {
            ensure_none(origin)?;
            TargetMinGasPrice::set(Some(target));
            Self::deposit_event(Event::TargetMinGasPriceSet(target));
        }
    }
}