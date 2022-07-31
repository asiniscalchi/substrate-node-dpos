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
	use frame_system::pallet_prelude::*;
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
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Map from all locked "stash" accounts to the controller account.
	#[pallet::storage]
	#[pallet::getter(fn bonded)]
	pub type Bonded<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, ()>;

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

			let stash_balance = T::Currency::free_balance(&stash);
			let value = value.min(stash_balance);
			T::Currency::set_lock(STAKING_ID, &stash, value, WithdrawReasons::all());

			<Bonded<T>>::insert(&stash, &());

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
			Self::deposit_event(Event::<T>::Unbonded(stash));

			Ok(())
		}
	}

	/// In this implementation `new_session(session)` must be called before `end_session(session-1)`
	/// i.e. the new session must be planned before the ending of the previous session.
	///
	/// Once the first new_session is planned, all session must start and then end in order, though
	/// some session can lag in between the newest session planned and the latest session started.
	impl<T: Config> pallet_session::SessionManager<T::AccountId> for Pallet<T> {
		fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
			log!(info, "planning new session {}", new_index);
			let mut result: Vec<T::AccountId> = Vec::new();
			for id in <Bonded<T>>::iter_keys() {
				result.push(id);
			}

			if result.is_empty() {
				return None
			}

			log!(info, "planning new session ids: {:?}", result);
			Some(result)
		}

		fn new_session_genesis(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
			log!(info, "planning new session {} at genesis", new_index);
			// CurrentPlannedSession::<T>::put(new_index);
			// Self::new_session(new_index, true)
			None
		}
		fn end_session(end_index: SessionIndex) {
			log!(info, "ending session {}", end_index);
			// Self::end_session(end_index)
		}
		fn start_session(start_index: SessionIndex) {
			log!(info, "starting session {}", start_index);
			// Self::start_session(start_index)
		}
	}
}
