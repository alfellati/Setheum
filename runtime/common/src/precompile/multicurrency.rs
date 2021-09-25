use frame_support::debug;
use module_evm::{Context, ExitError, ExitSucceed, Precompile};
use module_support::{AddressMapping as AddressMappingT, CurrencyIdMapping as CurrencyIdMappingT};
use sp_core::U256;
use sp_std::{fmt::Debug, marker::PhantomData, prelude::*, result};

use orml_traits::MultiCurrency as MultiCurrencyT;

use super::input::{Input, InputT};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use primitives::{Balance, CurrencyId};

/// The `MultiCurrency` impl precompile.
///
///
/// `input` data starts with `action` and `currency_id`.
///
/// Actions:
/// - Query total issuance.
/// - Query balance. Rest `input` bytes: `account_id`.
/// - Transfer. Rest `input` bytes: `from`, `to`, `amount`.
pub struct MultiCurrencyPrecompile<AccountId, AddressMapping, CurrencyIdMapping, MultiCurrency>(
	PhantomData<(AccountId, AddressMapping, CurrencyIdMapping, MultiCurrency)>,
);

#[primitives_proc_macro::generate_function_selector]
#[derive(RuntimeDebug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum Action {
	QueryName = "name()",
	QuerySymbol = "symbol()",
	QueryDecimals = "decimals()",
	QueryTotalIssuance = "totalSupply()",
	QueryBalance = "balanceOf(address)",
	Transfer = "transfer(address,address,uint256)",
}

impl<AccountId, AddressMapping, CurrencyIdMapping, MultiCurrency> Precompile
	for MultiCurrencyPrecompile<AccountId, AddressMapping, CurrencyIdMapping, MultiCurrency>
where
	AccountId: Debug + Clone,
	AddressMapping: AddressMappingT<AccountId>,
	CurrencyIdMapping: CurrencyIdMappingT,
	MultiCurrency: MultiCurrencyT<AccountId, Balance = Balance, CurrencyId = CurrencyId>,
{
	fn execute(
		input: &[u8],
		_target_gas: Option<u64>,
		context: &Context,
	) -> result::Result<(ExitSucceed, Vec<u8>, u64), ExitError> {
		//TODO: evaluate cost

		debug::debug!(target: "evm", "multicurrency: input: {:?}", input);

		let input = Input::<Action, AccountId, AddressMapping, CurrencyIdMapping>::new(input);

		let action = input.action()?;
		let currency_id = CurrencyIdMapping::decode_evm_address(context.caller)
			.ok_or_else(|| ExitError::Other("invalid currency id".into()))?;

		debug::debug!(target: "evm", "multicurrency: currency id: {:?}", currency_id);

		match action {
			Action::QueryName => {
				let name =
					CurrencyIdMapping::name(currency_id).ok_or_else(|| ExitError::Other("Get name failed".into()))?;
				debug::debug!(target: "evm", "multicurrency: name: {:?}", name);

				Ok((ExitSucceed::Returned, vec_u8_from_str(&name), 0))
			}
			Action::QuerySymbol => {
				let symbol = CurrencyIdMapping::symbol(currency_id)
					.ok_or_else(|| ExitError::Other("Get symbol failed".into()))?;
				debug::debug!(target: "evm", "multicurrency: symbol: {:?}", symbol);

				Ok((ExitSucceed::Returned, vec_u8_from_str(&symbol), 0))
			}
			Action::QueryDecimals => {
				let decimals = CurrencyIdMapping::decimals(currency_id)
					.ok_or_else(|| ExitError::Other("Get decimals failed".into()))?;
				debug::debug!(target: "evm", "multicurrency: decimals: {:?}", decimals);

				Ok((ExitSucceed::Returned, vec_u8_from_u8(decimals), 0))
			}
			Action::QueryTotalIssuance => {
				let total_issuance = vec_u8_from_balance(MultiCurrency::total_issuance(currency_id));
				debug::debug!(target: "evm", "multicurrency: total issuance: {:?}", total_issuance);

				Ok((ExitSucceed::Returned, total_issuance, 0))
			}
			Action::QueryBalance => {
				let who = input.account_id_at(1)?;
				debug::debug!(target: "evm", "multicurrency: who: {:?}", who);

				let balance = vec_u8_from_balance(MultiCurrency::total_balance(currency_id, &who));
				debug::debug!(target: "evm", "multicurrency: balance: {:?}", balance);

				Ok((ExitSucceed::Returned, balance, 0))
			}
			Action::Transfer => {
				let from = input.account_id_at(1)?;
				let to = input.account_id_at(2)?;
				let amount = input.balance_at(3)?;

				debug::debug!(target: "evm", "multicurrency: from: {:?}", from);
				debug::debug!(target: "evm", "multicurrency: to: {:?}", to);
				debug::debug!(target: "evm", "multicurrency: amount: {:?}", amount);

				MultiCurrency::transfer(currency_id, &from, &to, amount).map_err(|e| {
					let err_msg: &str = e.into();
					ExitError::Other(err_msg.into())
				})?;

				debug::debug!(target: "evm", "multicurrency: transfer success!");

				Ok((ExitSucceed::Returned, vec![], 0))
			}
		}
	}
}

fn vec_u8_from_balance(balance: Balance) -> Vec<u8> {
	let mut be_bytes = [0u8; 32];
	U256::from(balance).to_big_endian(&mut be_bytes[..]);
	be_bytes.to_vec()
}

fn vec_u8_from_u8(b: u8) -> Vec<u8> {
	let mut be_bytes = [0u8; 32];
	U256::from(b).to_big_endian(&mut be_bytes[..]);
	be_bytes.to_vec()
}

fn vec_u8_from_str(b: &[u8]) -> Vec<u8> {
	let mut be_bytes = [0u8; 32];
	U256::from_big_endian(b).to_big_endian(&mut be_bytes[..]);
	be_bytes.to_vec()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::precompile::mock::get_function_selector;

	#[test]
	fn function_selector_match() {
		assert_eq!(
			u32::from_be_bytes(get_function_selector("name()")),
			Into::<u32>::into(Action::QueryName)
		);

		assert_eq!(
			u32::from_be_bytes(get_function_selector("symbol()")),
			Into::<u32>::into(Action::QuerySymbol)
		);

		assert_eq!(
			u32::from_be_bytes(get_function_selector("decimals()")),
			Into::<u32>::into(Action::QueryDecimals)
		);

		assert_eq!(
			u32::from_be_bytes(get_function_selector("totalSupply()")),
			Into::<u32>::into(Action::QueryTotalIssuance)
		);

		assert_eq!(
			u32::from_be_bytes(get_function_selector("balanceOf(address)")),
			Into::<u32>::into(Action::QueryBalance)
		);

		assert_eq!(
			u32::from_be_bytes(get_function_selector("transfer(address,address,uint256)")),
			Into::<u32>::into(Action::Transfer)
		);
	}
}
