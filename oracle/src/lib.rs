#![cfg_attr(not(feature = "std"), no_std)]


use codec::{Decode, Encode};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use frame_support::{
	pallet_prelude::*,
	traits::{Get, Time, Currency},
	weights::{Pays, Weight},
	Parameter
};
use frame_system::{pallet_prelude::*};
use sp_std::marker;
use sp_std::{prelude::*, vec};

pub use orml_traits::{CombineData, DataFeeder, DataProvider, DataProviderExtended, OnNewData};

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
#[cfg(any(feature = "runtime-benchmarks", test))]
mod benchmarking;

pub(crate) type MomentOf<T, I = ()> = <<T as Config<I>>::Time as Time>::Moment;
pub(crate) type TimestampedValueOf<T, I = ()> = TimestampedValue<<T as Config<I>>::FeedValue, MomentOf<T, I>>;
pub(crate) type BalanceOf<T, I = ()> =
<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Clone, Copy, Ord, PartialOrd)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TimestampedValue<Value, Moment> {
	pub value: Value,
	pub timestamp: Moment,
}

//TODO: Should we deduct fee from callers?
#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;
		/// Time provider
		type Time: Time;
		/// The data key type
		type FeedKey: Parameter + Member;
		/// The data value type
		type FeedValue: Parameter + Member + Ord;
		/// Provide the implementation to combine raw values to produce
		/// aggregated value
		type CombineData: CombineData<Self::FeedKey, TimestampedValueOf<Self, I>>;
		/// Interface used for balance transfers.
		type Currency: Currency<Self::AccountId>;
		/// Fee pay for feeders
		#[pallet::constant]
		type OracleFee: Get<BalanceOf<Self, I>>;
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Sender does not have permission
		NoPermission,
		/// Feeder has already feeded at this block
		AlreadyFeeded,
		/// Already a feeder
		AlreadyFeeder,
		/// Not a feeder
		NotFeeder
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// New feed data is submitted. [sender, values]
		NewFeedData(T::AccountId, Vec<(T::FeedKey, T::FeedValue)>),
		/// New feeder is elected. [feeder]
		NewFeederElected(T::AccountId),
		/// New feeder is elected. [feeder, fee]
		FeeSetted(T::AccountId, BalanceOf<T, I>),
		/// Remove a feeder. [feeder]
		RemoveFeeder(T::AccountId),
	}

	#[pallet::storage]
	#[pallet::getter(fn feeders)]
	pub type Feeders<T: Config<I>, I: 'static = ()> =
	StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn has_feeded)]
	pub type HasFeeded<T: Config<I>, I: 'static = ()> =
	StorageMap<_, Twox64Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_combined)]
	pub type IsCombined<T: Config<I>, I: 'static = ()> =
	StorageMap<_, Twox64Concat, T::FeedKey, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn fees)]
	pub type Fees<T: Config<I>, I: 'static = ()> =
	StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T, I>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn all_value)]
	pub type AllValue<T: Config<I>, I: 'static = ()> =
	StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::FeedKey, TimestampedValueOf<T, I>>;

	/// Combined value, may not be up to date
	#[pallet::storage]
	#[pallet::getter(fn values)]
	pub type Values<T: Config<I>, I: 'static = ()> =
	StorageMap<_, Twox64Concat, T::FeedKey, TimestampedValueOf<T, I>>;

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<T::BlockNumber> for Pallet<T, I> {
		/// `on_initialize` to return the weight used in `on_finalize`.
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			10_000
		}

		fn on_finalize(_n: T::BlockNumber) {
			// cleanup for next block
			<HasFeeded<T, I>>::remove_all(None);
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		pub feeders: Vec<T::AccountId>,
		pub _phantom: marker::PhantomData<I>
	}

	#[cfg(feature = "std")]
	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			GenesisConfig { feeders: vec![], _phantom: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
		fn build(&self) {
			<Feeders<T, I>>::put(&self.feeders);
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::weight(0)]
		pub fn elect_feeder(
			origin: OriginFor<T>,
			feeder: T::AccountId
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let mut feeders = <Feeders<T, I>>::get();
			let location = feeders.binary_search(&feeder).err().ok_or(Error::<T, I>::AlreadyFeeder)?;
			feeders.insert(location, feeder.clone());

			<Feeders<T, I>>::put(feeders);

			Self::deposit_event(Event::NewFeederElected(feeder));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn remove_feeder(
			origin: OriginFor<T>,
			feeder: T::AccountId
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let mut feeders = <Feeders<T, I>>::get();
			let location = feeders.binary_search(&feeder).ok().ok_or(Error::<T, I>::NotFeeder)?;
			feeders.remove(location);

			<Feeders<T, I>>::put(feeders);
			<AllValue<T, I>>::remove_prefix(&feeder, None);

			Self::deposit_event(Event::RemoveFeeder(feeder));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn feed_values(
			origin: OriginFor<T>,
			values: Vec<(T::FeedKey, T::FeedValue)>
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let feeders = <Feeders<T, I>>::get();
			let _ = feeders.binary_search(&who).ok().ok_or(Error::<T, I>::NoPermission)?;

			if <HasFeeded<T, I>>::get(&who) {
				Err(Error::<T, I>::AlreadyFeeded)?;
			}

			Self::do_feed_values(who.clone(), values)?;

			Ok(Pays::No.into())
		}

		#[pallet::weight(0)]
		pub fn set_fee(
			origin: OriginFor<T>,
			#[pallet::compact] fee: BalanceOf<T, I>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let feeders = <Feeders<T, I>>::get();
			let _ = feeders.binary_search(&who).ok().ok_or(Error::<T, I>::NoPermission)?;

			<Fees<T, I>>::insert(&who, fee);
			Self::deposit_event(Event::FeeSetted(who, fee));

			Ok(().into())
		}
	}
}

impl <T: Config<I>, I: 'static> Pallet<T, I> {
	pub fn do_feed_values(who: T::AccountId, values: Vec<(T::FeedKey, T::FeedValue)>) -> DispatchResult {
		let now = T::Time::now();
		for (key, value) in &values {
			let timestamped_value =  TimestampedValue {
				value: value.clone(),
				timestamp: now
			};
			<IsCombined<T, I>>::remove(&key);
			<AllValue<T, I>>::insert(&who, &key, timestamped_value)
		}
		<HasFeeded<T, I>>::insert(&who, true);

		Self::deposit_event(Event::NewFeedData(who, values));
		Ok(())
	}

	pub fn get(
		key: &<T as Config<I>>::FeedKey,
	) -> Option<TimestampedValueOf<T, I>> {
		if <IsCombined<T, I>>::get(&key) {
			<Values<T, I>>::get(key)
		} else {
			let timestamped = Self::combine(key)?;
			<Values<T, I>>::insert(key, timestamped.clone());
			IsCombined::<T, I>::insert(key, true);
			Some(timestamped)
		}
	}

	pub fn get_concrete(
		key: &<T as Config<I>>::FeedKey,
		feeder: T::AccountId
	) -> Option<TimestampedValueOf<T, I>> {
		<AllValue<T, I>>::get(feeder, key)
	}

	fn get_values(key: &T::FeedKey) -> Vec<TimestampedValueOf<T, I>> {
		let mut feeders = <Feeders<T, I>>::get();
		feeders.sort();

		 feeders
			.iter()
			.filter_map(|x| <AllValue<T, I>>::get(x, key))
			.collect()
	}

	/// Returns fresh combined value if has update, or latest combined
	/// value.
	pub fn get_no_op(key: &T::FeedKey) -> Option<TimestampedValueOf<T, I>> {
		if Self::is_combined(key) {
			Self::values(key)
		} else {
			Self::combine(key)
		}
	}

	fn get_all_values() -> Vec<(T::FeedKey, Option<TimestampedValueOf<T, I>>)> {
		<Values<T, I>>::iter()
			.map(|(key, _)| key)
			.map(|key| {
				let v = Self::get_no_op(&key);
				(key, v)
			})
			.collect()
	}

	pub fn combine(
		key: &<T as Config<I>>::FeedKey,
	) -> Option<TimestampedValueOf<T, I>> {
		T::CombineData::combine_data(key, Self::get_values(&key), Self::values(&key))
	}
}

impl<T: Config<I>, I: 'static> DataProvider<T::FeedKey, TimestampedValueOf<T, I>> for Pallet<T, I> {
	fn get(key: &T::FeedKey) -> Option<TimestampedValueOf<T, I>> {
		Self::get(key)
	}
}

impl<T: Config<I>, I: 'static> DataProviderExtended<T::FeedKey, T::AccountId, TimestampedValueOf<T, I>> for Pallet<T, I> {
	fn get_polkafoundry(key: &T::FeedKey, feeder: T::AccountId) -> Option<TimestampedValueOf<T, I>> {
		Self::get_concrete(key, feeder)
	}

	fn get_concrete(key: &T::FeedKey, feeder: T::AccountId) -> Option<TimestampedValueOf<T, I>> {
		Self::get_concrete(key, feeder)
	}

	fn get_all_values() -> Vec<(T::FeedKey, Option<TimestampedValueOf<T, I>>)> {
		Self::get_all_values()
	}
}

impl<T: Config<I>, I: 'static> DataFeeder<T::FeedKey, T::FeedValue, T::AccountId> for Pallet<T, I> {
	fn feed_value(who: T::AccountId, key: T::FeedKey, value: T::FeedValue) -> DispatchResult {
		Self::do_feed_values(who, vec![(key, value)])?;
		Ok(())
	}
}

/// Sort by value and returns median timestamped value.
/// Returns prev_value if not enough valid values.
pub struct DefaultCombineData<T, MinimumCount, ExpiresIn, I = ()>(marker::PhantomData<(T, I, MinimumCount, ExpiresIn)>);

impl<T, I, MinimumCount, ExpiresIn> CombineData<<T as Config<I>>::FeedKey, TimestampedValueOf<T, I>>
for DefaultCombineData<T, MinimumCount, ExpiresIn, I>
	where
		T: Config<I>,
		I: 'static,
		MinimumCount: Get<u32>,
		ExpiresIn: Get<MomentOf<T, I>>,
{
	fn combine_data(
		_key: &<T as Config<I>>::FeedKey,
		mut values: Vec<TimestampedValueOf<T, I>>,
		prev_value: Option<TimestampedValueOf<T, I>>,
	) -> Option<TimestampedValueOf<T, I>> {
		let expires_in = ExpiresIn::get();
		let now = T::Time::now();

		values.retain(|x| x.timestamp + expires_in > now);

		let count = values.len() as u32;
		let minimum_count = MinimumCount::get();
		if count < minimum_count || count == 0 {
			return prev_value;
		}

		let mid_index = count / 2;

		// Won't panic as `values` ensured not empty.
		let (_, value, _) = values.select_nth_unstable_by(mid_index as usize, |a, b| a.value.cmp(&b.value));

		Some(value.clone())
	}
}
