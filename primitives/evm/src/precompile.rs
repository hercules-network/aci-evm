use evm::{Context, ExitError, ExitSucceed};
use impl_trait_for_tuples::impl_for_tuples;
use sp_core::H160;
use sp_std::vec::Vec;

pub trait PrecompileSet {
    fn execute(
        address: H160,
        input: &[u8],
        target_gas: Option<u64>,
        context: &Context,
    ) -> Option<core::result::Result<(ExitSucceed, Vec<u8>, u64), ExitError>>;
}

pub trait Precompile {
    fn execute(
        input: &[u8],
        target_gas: Option<u64>,
        context: &Context,
    ) -> core::result::Result<(ExitSucceed, Vec<u8>, u64), ExitError>;
}

#[impl_for_tuples(16)]
#[tuple_types_no_default_trait_bound]
impl PrecompileSet for Tuple {
    for_tuples!( where #( Tuple: Precompile )* );
    fn execute(
        address: H160,
        input: &[u8],
        target_gas: Option<u64>,
        context: &Context,
    ) -> Option<core::result::Result<(ExitSucceed, Vec<u8>, u64), ExitError>> {
        let mut index = 0;
        for_tuples!( #(
			index += 1;
			if address == H160::from_low_u64_be(index) {
				return Some(Tuple::execute(input, target_gas, context))
			}
		)* );
        None
    }
}

pub trait LinearCostPrecompile {
    const BASE: u64;
    const WORD: u64;
    fn execute(input: &[u8], cost: u64) -> core::result::Result<(ExitSucceed, Vec<u8>), ExitError>;
}

impl<T: LinearCostPrecompile> Precompile for T {
    fn execute(
        input: &[u8],
        target_gas: Option<u64>,
        _: &Context,
    ) -> core::result::Result<(ExitSucceed, Vec<u8>, u64), ExitError> {
        let cost = ensure_linear_cost(target_gas, input.len() as u64, T::BASE, T::WORD)?;
        let (succeed, out) = T::execute(input, cost)?;
        Ok((succeed, out, cost))
    }
}

fn ensure_linear_cost(
    target_gas: Option<u64>,
    len: u64,
    base: u64,
    word: u64,
) -> Result<u64, ExitError> {
    let cost = base
        .checked_add(
            word.checked_mul(len.saturating_add(31) / 32)
                .ok_or(ExitError::OutOfGas)?,
        )
        .ok_or(ExitError::OutOfGas)?;
    if let Some(target_gas) = target_gas {
        if cost > target_gas {
            return Err(ExitError::OutOfGas);
        }
    }
    Ok(cost)
}