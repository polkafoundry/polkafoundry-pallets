#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::pallet;
pub use pallet::*;

#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

use frame_support::{
	pallet_prelude::*,
	traits::{
		Currency, ExistenceRequirement::AllowDeath, IsType, LockIdentifier, LockableCurrency, Time, WithdrawReasons,
	},
	PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
	traits::{AccountIdConversion, CheckedSub, Saturating, Zero},
	Perbill,
};
use sp_std::{convert::From, vec::Vec};

const REDKITE_ID: LockIdentifier = *b"redkite ";
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type MomentOf<T> = <<T as Config>::Time as Time>::Moment;

#[pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type PalletId: Get<PalletId>;

		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

		type Time: Time;
	}

	#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug)]
	pub struct PoolInfo<T: Config> {
		pub token: u32, // address
		pub open_time: MomentOf<T>,
		pub close_time: MomentOf<T>,
		pub offered_currency: u32,
		pub funding_wallet: T::AccountId,
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
	pub struct SettingStruct<T: Config> {
		pub tier_minimum_points: Vec<(Tier, BalanceOf<T>)>,
	}

	impl<T> SettingStruct<T>
	where
		T: Config,
	{
		pub fn update_tier_system(&mut self, new_tier_points: Vec<BalanceOf<T>>) {
			if new_tier_points.len() != 4 {
				return;
			}

			let mut tiers: Vec<(Tier, BalanceOf<T>)> = Vec::new();
			tiers.push((Tier::Dove, new_tier_points[0]));
			tiers.push((Tier::Hawk, new_tier_points[1]));
			tiers.push((Tier::Eagle, new_tier_points[2]));
			tiers.push((Tier::Phoenix, new_tier_points[3]));

			self.tier_minimum_points = tiers;
		}
	}

	impl<T: Config> Default for SettingStruct<T> {
		fn default() -> Self {
			Self {
				tier_minimum_points: Vec::default(),
			}
		}
	}

	#[derive(Default, Clone, Encode, Decode, RuntimeDebug)]
	pub struct CurrencyInfo {
		pub offered_currency_decimals: u32,
		pub offered_currency_rate: u32,
	}

	#[derive(Default, Clone, Encode, Decode, RuntimeDebug)]
	pub struct UserWinnerInfo<T: Config> {
		pub max_purchased: BalanceOf<T>,
		pub min_purchased: BalanceOf<T>,
		pub purchased: BalanceOf<T>,
		pub claimed: BalanceOf<T>,
	}

	impl<T> UserWinnerInfo<T>
	where
		T: Config,
	{
		pub fn default_with_max_purchased(amount: BalanceOf<T>) -> Self {
			Self {
				min_purchased: Zero::zero(),
				max_purchased: amount,
				purchased: Zero::zero(),
				claimed: Zero::zero(),
			}
		}
	}

	/// Redkite Tier System
	#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug)]
	pub enum Tier {
		/// Phoenix
		Phoenix,
		/// Eagle
		Eagle,
		/// Hawk
		Hawk,
		/// Dove
		Dove,
		/// None
		None,
	}

	/// Permission System
	#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug)]
	pub enum Permission {
		/// Administrator
		Administrator,
		/// Operation
		Operator,
	}

	#[derive(Default, Clone, Encode, Decode, RuntimeDebug)]
	pub struct UserInfo<T: Config> {
		pub total_staked: BalanceOf<T>,
		pub bonus: BalanceOf<T>,
		pub last_staked_at: MomentOf<T>,
	}

	impl<T> UserInfo<T>
	where
		T: Config,
	{
		pub fn stake(&mut self, amount: BalanceOf<T>, now: MomentOf<T>) {
			self.total_staked = self.total_staked.saturating_add(amount);
			self.last_staked_at = now;
		}

		pub fn un_stake(&mut self, amount: BalanceOf<T>, now: MomentOf<T>) {
			if self.total_staked >= amount {
				self.total_staked = self.total_staked.saturating_sub(amount);
				self.last_staked_at = now;
			}
		}

		pub fn set_bonus(&mut self, amount: BalanceOf<T>) {
			self.bonus = amount;
		}

		pub fn point(self) -> BalanceOf<T> {
			self.total_staked.saturating_add(self.bonus)
		}

		pub fn tier(self) -> Tier {
			let setting = Settings::<T>::get();
			let point = self.point();
			let mut result = Tier::None;

			for (tier, amount) in setting.tier_minimum_points {
				if point >= amount {
					result = tier
				}
			}

			result
		}

		fn default() -> Self {
			Self {
				total_staked: Zero::zero(),
				bonus: Zero::zero(),
				last_staked_at: Zero::zero(),
			}
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// - Add setting for calculate Tier
	// - Add Bonus external system point (~~ ePKF)
	// - Convert token type: u32 --> tokens::CurrencyId

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn init_pool(
			origin: OriginFor<T>,
			pool_id: u32,
			token: u32,
			duration: MomentOf<T>,
			open_time: MomentOf<T>,
			offered_currency: u32,
			funding_wallet: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_admin(who), Error::<T>::InvalidPermission);

			let close_time = open_time.saturating_add(duration);
			Pools::<T>::insert(
				pool_id,
				PoolInfo {
					token: token.clone(),
					open_time,
					close_time,
					offered_currency: offered_currency.clone(),
					funding_wallet: funding_wallet.clone(),
				},
			);

			Self::deposit_event(Event::PoolChanged(
				pool_id,
				token,
				open_time,
				close_time,
				offered_currency,
				funding_wallet,
			));
			Ok(Default::default())
		}

		#[pallet::weight(0)]
		pub fn set_pool_winners(
			origin: OriginFor<T>,
			pool_id: u32,
			winners: Vec<(T::AccountId, BalanceOf<T>)>, // (account,max_amount)
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_admin(who), Error::<T>::InvalidPermission);
			// TODO: clear the old winners

			for (who, amount) in winners {
				let new_user_winner_info = UserWinnerInfo::default_with_max_purchased(amount);
				Winners::<T>::insert(pool_id, &who, new_user_winner_info);
			}

			Ok(Default::default())
		}

		#[pallet::weight(0)]
		pub fn set_offered_currency_rate_and_decimals(
			origin: OriginFor<T>,
			offered_currency: u32,
			offered_currency_decimals: u32,
			offered_currency_rate: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_admin(who), Error::<T>::InvalidPermission);

			OfferedCurrencies::<T>::insert(
				offered_currency,
				CurrencyInfo {
					offered_currency_decimals,
					offered_currency_rate,
				},
			);
			Self::deposit_event(Event::OfferedCurrenciesChanged(
				offered_currency,
				offered_currency_decimals,
				offered_currency_rate,
			));
			Ok(Default::default())
		}

		#[pallet::weight(0)]
		pub fn set_close_time(
			origin: OriginFor<T>,
			pool_id: u32,
			close_time: MomentOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_admin(who), Error::<T>::InvalidPermission);

			let mut pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			pool.close_time = close_time;

			Pools::<T>::insert(pool_id, pool.clone());
			Self::deposit_event(Event::PoolChanged(
				pool_id,
				pool.token,
				pool.open_time,
				pool.close_time,
				pool.offered_currency,
				pool.funding_wallet,
			));
			Ok(Default::default())
		}

		#[pallet::weight(0)]
		pub fn set_open_time(origin: OriginFor<T>, pool_id: u32, open_time: MomentOf<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_admin(who), Error::<T>::InvalidPermission);

			let mut pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			pool.open_time = open_time;

			Pools::<T>::insert(pool_id, pool.clone());
			Self::deposit_event(Event::PoolChanged(
				pool_id,
				pool.token,
				pool.open_time,
				pool.close_time,
				pool.offered_currency,
				pool.funding_wallet,
			));
			Ok(Default::default())
		}

		#[pallet::weight(0)]
		pub fn buy_token(origin: OriginFor<T>, pool_id: u32, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
			let mut winner = Winners::<T>::get(pool_id, &who).ok_or(Error::<T>::WinnerNotFound)?;
			let rate = OfferedCurrencies::<T>::get(pool.token).ok_or(Error::<T>::RateNotFound)?;
			let token_amount = Perbill::from_rational(1, 10u32.saturating_pow(rate.offered_currency_decimals)) // 1/10^decimals
				.mul_floor(amount.saturating_mul(rate.offered_currency_rate.into()));

			let now = T::Time::now();
			ensure!(pool.open_time <= now && pool.close_time >= now, Error::<T>::PoolClosed);
			ensure!(
				winner.min_purchased < token_amount,
				Error::<T>::PurchaseAmountBelowMinimum
			);
			ensure!(
				winner.purchased.saturating_add(token_amount) <= winner.max_purchased,
				Error::<T>::PurchaseAmountAboveMaximum
			);

			let _ = T::Currency::transfer(&who, &pool.funding_wallet, amount, AllowDeath)
				.map_err(|_| Error::<T>::BuyTokenFailed)?;

			winner.purchased = winner.purchased.saturating_add(token_amount);
			Winners::<T>::insert(pool_id, &who, winner);
			Self::deposit_event(Event::TokenPurchased(pool_id, who, amount));

			Ok(Default::default())
		}

		#[pallet::weight(0)]
		pub fn claim_token(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let _ = ensure_signed(origin)?;
			// TODO: transfer token to users
			Ok(Default::default())
		}

		#[pallet::weight(0)]
		pub fn stake(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = T::Time::now();

			T::Currency::set_lock(REDKITE_ID, &who, amount, WithdrawReasons::all());
			let mut info = match RedkitePoints::<T>::get(&who) {
				Some(item) => item,
				None => UserInfo::default(),
			};
			info.stake(amount, now);
			RedkitePoints::<T>::insert(&who, info);
			Self::deposit_event(Event::UserStaked(who, amount, now));

			Ok(Default::default())
		}

		#[pallet::weight(0)]
		pub fn un_stake(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let now = T::Time::now();

			let mut info = RedkitePoints::<T>::get(&who).ok_or(Error::<T>::UserNotFound)?;
			ensure!(info.total_staked >= amount, Error::<T>::InsufficientBalance);

			T::Currency::remove_lock(REDKITE_ID, &who);
			info.un_stake(amount, now);
			RedkitePoints::<T>::insert(&who, info);
			Self::deposit_event(Event::UserUnStaked(who, amount, now));

			Ok(Default::default())
		}

		#[pallet::weight(0)]
		pub fn grant_administrators(origin: OriginFor<T>, accounts: Vec<T::AccountId>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_admin(who), Error::<T>::InvalidPermission);

			for account in accounts {
				PermissionsSystem::<T>::insert(&account, Permission::Administrator);
				Self::deposit_event(Event::GrantAdministrator(account));
			}
			Ok(Default::default())
		}

		#[pallet::weight(0)]
		pub fn grant_operators(origin: OriginFor<T>, accounts: Vec<T::AccountId>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_operator(who), Error::<T>::InvalidPermission);

			for account in accounts {
				PermissionsSystem::<T>::insert(&account, Permission::Operator);
				Self::deposit_event(Event::GrantOperator(account));
			}
			Ok(Default::default())
		}

		#[pallet::weight(0)]
		pub fn set_bonus(
			origin: OriginFor<T>,
			accounts: Vec<(T::AccountId, BalanceOf<T>)>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_admin(who), Error::<T>::InvalidPermission);

			for (account, amount) in accounts {
				let mut info = match RedkitePoints::<T>::get(&account) {
					Some(item) => item,
					None => UserInfo::default(),
				};
				info.set_bonus(amount);
				RedkitePoints::<T>::insert(&account, info);
			}
			Ok(Default::default())
		}

		#[pallet::weight(0)]
		pub fn update_tier_setting(
			origin: OriginFor<T>,
			tier_minimum_points: Vec<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			ensure!(Self::is_admin(who), Error::<T>::InvalidPermission);
			// Dove, Hawk, Eagle, Phoenix
			ensure!(tier_minimum_points.len() == 4, Error::<T>::InvalidTierSetting);

			let mut setting = Settings::<T>::get();
			setting.update_tier_system(tier_minimum_points);
			Settings::<T>::put(setting);
			// Self::deposit_event(Event::UpdateSetting(setting.clone()));

			Ok(Default::default())
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub type Pools<T: Config> = StorageMap<_, Blake2_128Concat, u32, PoolInfo<T>>;

	#[pallet::storage]
	#[pallet::getter(fn winners)]
	pub type Winners<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, T::AccountId, UserWinnerInfo<T>>;

	#[pallet::storage]
	#[pallet::getter(fn offered_currencies)]
	pub type OfferedCurrencies<T: Config> = StorageMap<_, Blake2_128Concat, u32, CurrencyInfo>;

	#[pallet::storage]
	#[pallet::getter(fn redkite_points)]
	pub type RedkitePoints<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, UserInfo<T>>;

	#[pallet::storage]
	#[pallet::getter(fn permissions_system)]
	pub type PermissionsSystem<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Permission>;

	#[pallet::storage]
	#[pallet::getter(fn settings)]
	pub type Settings<T: Config> = StorageValue<_, SettingStruct<T>, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// Pool not found
		PoolNotFound,
		/// Pool closed
		PoolClosed,
		/// Winner not found
		WinnerNotFound,
		/// Exchange rate between Native token and token is not found
		RateNotFound,
		/// The amount of purchase below the minimum
		PurchaseAmountBelowMinimum,
		/// The amount of purchase above the maximum
		PurchaseAmountAboveMaximum,
		/// Token is staked failed
		StakeTokenFailed,
		/// Token is unstaked failed
		UnstakeTokenFailed,
		/// Token is bought failed
		BuyTokenFailed,
		/// User not found
		UserNotFound,
		/// Insufficient Balance
		InsufficientBalance,
		/// Invalid permisison
		InvalidPermission,
		/// Invalid tier system
		InvalidTierSetting,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		PoolChanged(u32, u32, MomentOf<T>, MomentOf<T>, u32, T::AccountId),
		OfferedCurrenciesChanged(u32, u32, u32),
		TokenPurchased(u32, T::AccountId, BalanceOf<T>),
		TokenClaimed(u32, u32, u32, u32),
		UserStaked(T::AccountId, BalanceOf<T>, MomentOf<T>),
		UserUnStaked(T::AccountId, BalanceOf<T>, MomentOf<T>),
		GrantAdministrator(T::AccountId),
		GrantOperator(T::AccountId),
		// UpdateSetting(SettingStruct<T>),
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub administrators: Vec<T::AccountId>,
		pub tiers: Vec<BalanceOf<T>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				administrators: Vec::default(),
				tiers: Vec::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for account in &self.administrators {
				PermissionsSystem::<T>::insert(&account, Permission::Administrator);
			}
			let mut setting = Settings::<T>::get();
			setting.update_tier_system(self.tiers.clone());
			Settings::<T>::put(setting);
		}
	}

	#[pallet::extra_constants]
	impl<T: Config> Pallet<T> {
		/// The account ID of the pallet.
		///
		/// This actually does computation. If you need to keep using it, then make sure you cache the
		/// value and only call this once.
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account()
		}

		pub fn pot() -> BalanceOf<T> {
			T::Currency::free_balance(&Self::account_id())
				.checked_sub(&T::Currency::minimum_balance())
				.unwrap_or_else(Zero::zero)
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_offered_currency_rate(address: u32) -> u32 {
			match OfferedCurrencies::<T>::get(address) {
				Some(item) => item.offered_currency_rate,
				None => 0,
			}
		}

		pub fn get_offered_currency_decimals(address: u32) -> u32 {
			match OfferedCurrencies::<T>::get(address) {
				Some(item) => item.offered_currency_decimals,
				None => 0,
			}
		}

		pub fn is_admin(account: T::AccountId) -> bool {
			match PermissionsSystem::<T>::get(&account) {
				Some(item) => item == Permission::Administrator,
				None => false,
			}
		}

		pub fn is_operator(account: T::AccountId) -> bool {
			match PermissionsSystem::<T>::get(&account) {
				Some(item) => item == Permission::Operator,
				None => false,
			}
		}
	}
}
