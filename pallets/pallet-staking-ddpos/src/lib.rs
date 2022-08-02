#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub(crate) const LOG_TARGET: &str = "runtime::dpos";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: crate::LOG_TARGET,
			concat!("[{:?}] 💸 ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, LockIdentifier, LockableCurrency, WithdrawReasons};
	use frame_system::{ensure_root, pallet_prelude::*};
	use sp_staking::SessionIndex;
	use sp_std::vec::Vec;

	const STAKING_ID: LockIdentifier = *b"staking ";

	/// The balance type of this pallet.
	pub type BalanceOf<T> = <T as Config>::CurrencyBalance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The staking balance.
		type Currency: LockableCurrency<
			Self::AccountId,
			Moment = Self::BlockNumber,
			Balance = Self::CurrencyBalance,
		>;
		/// Just the `Currency::Balance` type; we have this item to allow us to constrain it to
		/// `From<u64>`.
		type CurrencyBalance: sp_runtime::traits::AtLeast32BitUnsigned
			+ codec::FullCodec
			+ Copy
			+ MaybeSerializeDeserialize
			+ sp_std::fmt::Debug
			+ Default
			+ From<u64>
			+ TypeInfo
			+ MaxEncodedLen;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type MinimumValidatorCount: Get<u32>;

		#[pallet::constant]
		type MaximumValidatorCount: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Map from all locked "stash" accounts to the controller account.
	#[pallet::storage]
	#[pallet::getter(fn bonded)]
	pub type Bonded<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>>;

	/// Minimum number of staking participants before emergency conditions are imposed.
	#[pallet::storage]
	#[pallet::getter(fn minimum_validator_count)]
	pub type MinimumValidatorCount<T> =
		StorageValue<_, u32, ValueQuery, <T as Config>::MinimumValidatorCount>;

	/// Minimum number of staking participants before emergency conditions are imposed.
	#[pallet::storage]
	#[pallet::getter(fn maximum_validator_count)]
	pub type MaximumValidatorCount<T> =
		StorageValue<_, u32, ValueQuery, <T as Config>::MaximumValidatorCount>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An account has bonded this amount.
		Bonded(T::AccountId, BalanceOf<T>),
		/// An account has unbonded
		Unbonded(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Not a stash account.
		NotStash,
		/// Stash is already bonded.
		AlreadyBonded,
		/// Cannot have a validator or nominator role, with value less than the minimum defined by
		/// governance (see `MinValidatorBond` and `MinNominatorBond`). If unbonding is the
		/// intention, `chill` first to remove one's role as validator/nominator.
		InsufficientBond,
		BadState,
		/// Invalid number of validators.
		InvalidNumberOfValidators,

		AlreadyVoted,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		//TODO #[pallet::weight(T::WeightInfo::bond())]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn bond(
			origin: OriginFor<T>,
			#[pallet::compact] value: BalanceOf<T>,
		) -> DispatchResult {
			let stash = ensure_signed(origin)?;

			if <Bonded<T>>::contains_key(&stash) {
				return Err(Error::<T>::AlreadyBonded.into());
			}

			// Reject a bond which is considered to be _dust_.
			if value < T::Currency::minimum_balance() {
				return Err(Error::<T>::InsufficientBond.into());
			}

			frame_system::Pallet::<T>::inc_consumers(&stash).map_err(|_| Error::<T>::BadState)?;

			let stash_balance = T::Currency::free_balance(&stash);
			let value = value.min(stash_balance);
			T::Currency::set_lock(STAKING_ID, &stash, value, WithdrawReasons::all());

			<Bonded<T>>::insert(&stash, value);

			Self::deposit_event(Event::<T>::Bonded(stash, value));

			Ok(())
		}

		// TODO #[pallet::weight(T::WeightInfo::unbond())]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn unbond(origin: OriginFor<T>) -> DispatchResult {
			let stash = ensure_signed(origin)?;

			if None == <Bonded<T>>::take(&stash) {
				return Err(Error::<T>::NotStash.into());
			}

			T::Currency::remove_lock(STAKING_ID, &stash);

			frame_system::Pallet::<T>::dec_consumers(&stash);

			Self::deposit_event(Event::<T>::Unbonded(stash));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn set_minimum_validator_count(origin: OriginFor<T>, value: u32) -> DispatchResult {
			ensure_root(origin)?;
			if value == 0 || value > <MaximumValidatorCount<T>>::get() {
				return Err(Error::<T>::InvalidNumberOfValidators.into());
			}
			<MinimumValidatorCount<T>>::set(value);
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn set_maximum_validator_count(origin: OriginFor<T>, value: u32) -> DispatchResult {
			ensure_root(origin)?;
			if value < <MinimumValidatorCount<T>>::get() {
				return Err(Error::<T>::InvalidNumberOfValidators.into());
			}
			<MaximumValidatorCount<T>>::set(value);
			Ok(())
		}

// #[pallet::weight(T::WeightInfo::nominate(targets.len() as u32))]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn vote(
			origin: OriginFor<T>,
			target: T::AccountId,
		) -> DispatchResult {
			let controller = ensure_signed(origin)?;
			 
			Ok(())
		}
	}

	impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
		fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
			log!(debug, "planning new session {}", new_index);

			let min_validator_count = <MinimumValidatorCount<T>>::get();
			let max_validator_count = <MaximumValidatorCount<T>>::get();

			let mut validators: Vec<(T::AccountId, BalanceOf<T>)> = <Bonded<T>>::iter().collect();
			if validators.len() < min_validator_count as usize {
				log!(
					warn,
					"validators count {} less than the minimum {} ... skip",
					validators.len(),
					min_validator_count
				);
				return None;
			}

			validators.sort_by(|a, b| b.1.cmp(&a.1));
			validators.truncate(max_validator_count as usize);

			let mut winners: Vec<T::AccountId> = Vec::new();
			for i in validators {
				winners.push(i.0);
			}

			Some(winners)
		}

		fn end_session(end_index: SessionIndex) {
			log!(debug, "ending session {}", end_index);
		}

		fn start_session(start_index: SessionIndex) {
			log!(debug, "starting session {}", start_index);
		}
	}
}
