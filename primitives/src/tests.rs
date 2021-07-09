use super::*;
use crate::currency::TokenInfo;
use frame_support::assert_ok;
use std::{
	convert::{TryFrom, TryInto},
	str::FromStr,
};

#[test]
fn currency_id_try_from_vec_u8_works() {
	assert_ok!(
		"PKF".as_bytes().to_vec().try_into(),
		CurrencyId::Token(TokenSymbol::PKF)
	);
}
