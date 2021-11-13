#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;
	use codec::{Encode, Decode};
	use sp_runtime::traits::StaticLookup;

	#[derive(Debug, Clone, PartialEq, Default, Encode, Decode, scale_info::TypeInfo)]
	pub struct Order<AccountId> {
		id: u128,
		user: AccountId,
		products: Vec<(Vec<u8>, u32, u32)>,
		total: u32,
		date: Vec<u8>,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config + pallet_users::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	pub type Orders<T: Config> = StorageMap<_, Twox64Concat, u128, Order<T::AccountId>>;

	#[pallet::storage]
	pub type OrdersByUser<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<Order<T::AccountId>>>;

	#[pallet::storage]
	pub (super) type OrderCount<T> = StorageValue<_, u128>;

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
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn purchase(
			origin: OriginFor<T>, 
			products: Vec<(Vec<u8>, u32, u32)>,
			owner: T::AccountId,
			date: Vec<u8>) -> DispatchResult {
				let who = ensure_signed(origin.clone())?;
				let total = Self::get_purchase_total(&products);
			  	pallet_balances::Pallet::<T>::transfer(origin, T::Lookup::unlookup(owner), total.into());
				let count = OrderCount::<T>::get().unwrap_or(0);
				Orders::<T>::insert(count.clone(), Order {
					id: count.clone(),
					user: who.clone(),
					products,
					total,
					date,
				});
				let order = Orders::<T>::get(count.clone()).unwrap_or(Default::default());
				Self::add_order_to_user_orders(&order, &who);
				OrderCount::<T>::put(count + 1);
				Ok(())
			}
	}

	impl<T: Config> Pallet<T> {
		
		fn add_order_to_user_orders(order: &Order<T::AccountId>, user: &T::AccountId) {
			let mut _user = pallet_users::Pallet::<T>::get_user(user);
			let mut orders = OrdersByUser::<T>::get(user).unwrap_or(Default::default());
			_user.total_orders += 1;
			let _order = order.clone();
			orders.push(_order);
			pallet_users::Pallet::<T>::insert_user(&user, &_user);
			OrdersByUser::<T>::insert(user, orders);
		}

		fn get_purchase_total(products: &Vec<(Vec<u8>, u32, u32)>) -> u32 {
			let mut total: u32 = 0;
			for i in products {
				total += i.1 * i.2;
			}
			total
		}
	}
}
