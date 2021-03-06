#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;
use ap_evm::Precompile;
use evm::{ExitSucceed, ExitError, Context};
use frame_support::{dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo}, weights::{Pays, DispatchClass}};
use pallet_evm::{AddressMapping, GasWeightMapping};
use codec::Decode;

pub struct Dispatch<T: pallet_evm::Config> {
	_marker: PhantomData<T>,
}

impl<T> Precompile for Dispatch<T> where
	T: pallet_evm::Config,
	T::Call: Dispatchable<PostInfo=PostDispatchInfo> + GetDispatchInfo + Decode,
	<T::Call as Dispatchable>::Origin: From<Option<T::AccountId>>,
{
	fn execute(
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
	) -> core::result::Result<(ExitSucceed, Vec<u8>, u64), ExitError> {
		let call = T::Call::decode(&mut &input[..]).map_err(|_| ExitError::Other("decode failed".into()))?;
		let info = call.get_dispatch_info();

		let valid_call = info.pays_fee == Pays::Yes && info.class == DispatchClass::Normal;
		if !valid_call {
			return Err(ExitError::Other("invalid call".into()))
		}

		if let Some(gas) = target_gas {
			let valid_weight = info.weight <= T::GasWeightMapping::gas_to_weight(gas);
			if !valid_weight {
				return Err(ExitError::OutOfGas)
			}
		}

		let origin = T::AddressMapping::into_account_id(context.caller);

		match call.dispatch(Some(origin).into()) {
			Ok(post_info) => {
				let cost = T::GasWeightMapping::weight_to_gas(post_info.actual_weight.unwrap_or(info.weight));
				Ok((ExitSucceed::Stopped, Default::default(), cost))
			},
			Err(_) => Err(ExitError::Other("dispatch execution failed".into())),
		}
	}
}
