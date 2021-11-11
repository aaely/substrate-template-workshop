#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;
	use codec::{Encode, Decode};

	#[derive(Debug, Clone, PartialEq, Default, Encode, Decode, scale_info::TypeInfo)]
	pub struct User<AccountId> {
		pub address: AccountId,
		fname: Vec<u8>,
		lname: Vec<u8>,
		phone: Vec<u8>,
		email: Vec<u8>,
		pub handle: Vec<u8>,
		handle_id: u128,
		bio: Vec<u8>,
		website: Vec<u8>,
		profile_image: Vec<u8>,
		pub total_orders: u32,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	pub type UserCount<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn get_admin)]
	pub(super) type Admins<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_user_access)]
	pub(super) type UserAccess<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_user)]
	pub(super) type Users<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, User<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_user_by_handle)]
	pub(super) type UserByHandle<T: Config> = StorageMap<_, Twox64Concat, u128, User<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_owner)]
	pub(super) type Owner<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn get_user_handle_availability)]
	pub type UserHandleAvailability<T> = StorageMap<_, Twox64Concat, u128, bool>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		UserAlreadyExists,
		InsufficientPriv,
		HandleAlreadyExists,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
		pub fn new_user(
			origin: OriginFor<T>, 
			fname: Vec<u8>, 
			lname: Vec<u8>, 
			phone: Vec<u8>, 
			email: Vec<u8>,
			handle: Vec<u8>,
			handle_id: u128,
			bio: Vec<u8>,
			website: Vec<u8>,
			profile_image: Vec<u8>) -> DispatchResult {
				let who = ensure_signed(origin)?;
				ensure!(!Self::check_duplicate_user(&who), Error::<T>::UserAlreadyExists);
				ensure!(!Self::get_user_handle_availability(&handle_id).unwrap_or(Default::default()), Error::<T>::HandleAlreadyExists);
				let count = UserCount::<T>::get().unwrap_or(0);
				Self::register_user(&who);
				Users::<T>::insert(who.clone(), User {
					address: who.clone(),
					fname,
					lname,
					phone,
					email,
					handle,
					handle_id,
					bio,
					website,
					profile_image,
					total_orders: 0,
				});
				UserHandleAvailability::<T>::insert(handle_id, true);
				let new_user = Users::<T>::get(who.clone());
				Self::initiate_user(&new_user);
				UserCount::<T>::put(count + 1);
				Ok(())
			}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
		pub fn edit_user(
			origin: OriginFor<T>, 
			fname: Vec<u8>, 
			lname: Vec<u8>, 
			phone: Vec<u8>, 
			email: Vec<u8>,
			handle: Vec<u8>,
			bio: Vec<u8>,
			website: Vec<u8>,
			handle_id: u128,
			profile_image: Vec<u8>,
			total_orders: u32) -> DispatchResult {
				let who = ensure_signed(origin)?;
				ensure!(!Self::check_is_user(&who), Error::<T>::InsufficientPriv);				
				Users::<T>::insert(who.clone(), User {
					address: who.clone(),
					fname,
					lname,
					phone,
					email,
					handle,
					handle_id,
					bio,
					website,
					profile_image,
					total_orders,
				});
				UserHandleAvailability::<T>::insert(handle_id, true);
				let edited_user = Users::<T>::get(who.clone());
				Self::initiate_user(&edited_user);
				Ok(())
			}
	}
	
	impl<T: Config> Pallet<T> {
		pub fn check_duplicate_user(id: &T::AccountId) -> bool {
			let user = Self::get_user(id);
			if user.fname.len() > 0 {
				true
			} else {
				false
			}
		}

		fn initiate_user(user: &User<T::AccountId>) {
			UserByHandle::<T>::insert(user.handle_id, user);
		}

		pub fn get_profile_image_by_handle(
			handle: &u128, 
		) -> Vec<u8> {
			let _user = Self::get_user_by_handle(handle);
			_user.profile_image
		}

		pub fn check_duplicate_handle(id: &u128) -> bool {
			UserHandleAvailability::<T>::get(id).unwrap_or(false)
		}

		pub fn check_is_user(id: &T::AccountId) -> bool {
			let user = Users::<T>::get(id);
			if user.address.eq(id) {
				true
			} else {
				false
			}
		}

		pub fn insert_user(id: &T::AccountId, user: &User<T::AccountId>) {
			Users::<T>::insert(id, user);
		}

		fn register_user(id: &T::AccountId) {
			UserAccess::<T>::insert(id, true);
		}
	}
}
