#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;


#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::{pallet_prelude::*, ensure_signed};
	use codec::{Encode, Decode};
	use sp_std::prelude::*;

	#[derive(Debug, Clone, PartialEq, Default, Encode, Decode, scale_info::TypeInfo)]
	pub struct Record<AccountId> {
		id: u128,
		original_creator: AccountId,
		owner: AccountId,
		ip_addresses: Vec<Vec<u8>>,
		last_price: u32,
		current_price: u32,
		for_sale: bool,
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

		
	#[pallet::storage]
	#[pallet::getter(fn records_by_owner)]
	pub type RecordsByOwner<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<Record<T::AccountId>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn record_by_id)]
	pub type RecordById<T: Config> = StorageMap<_, Twox64Concat, u128, Record<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_exist)]
	pub type RecordExists<T: Config> = StorageMap<_, Twox64Concat, u128, bool, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewRecord(Record<T::AccountId>),
		RecordTransfer(T::AccountId, u128),
	}

	#[pallet::error]
	pub enum Error<T> {
		AlreadyInUse,
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(3))]
		pub fn register_name(
			origin: OriginFor<T>, 
			id: u128,
			ip: Vec<Vec<u8>>,
			last_price: u32,
			for_sale: bool,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			ensure!(!Self::check_duplicate(&id), Error::<T>::AlreadyInUse);
			RecordById::<T>::insert(id.clone(), Record {
				id: id.clone(),
				original_creator: signer.clone(),
				owner: signer.clone(),
				ip_addresses: ip,
				last_price,
				current_price: 0,
				for_sale
			});
			let record = Self::record_by_id(id);
			Self::add_record(record, &signer);
			Self::deposit_event(Event::<T>::NewRecord(Self::record_by_id(id)));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn change_owner(
			origin: OriginFor<T>,
			id: u128,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			Self::transfer_record_ownership(&id, &signer);
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn update_for_sale(
			origin: OriginFor<T>,
			id: u128,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			let mut record = Self::record_by_id(id.clone());
			record.for_sale = !record.for_sale;
			Self::update_record(&record, &id, &signer);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// Helper Fucntions
		fn check_duplicate(id: &u128) -> bool {
				Self::is_exist(id)
		}

		fn add_record(
			record: Record<T::AccountId>, 
			act: &T::AccountId
		) {
			
			let mut records = Self::records_by_owner(act);
			records.push(record.clone());
			RecordsByOwner::<T>::insert(act, records);
			RecordExists::<T>::insert(record.id, true);
		}

		fn transfer_record_ownership(id: &u128, new_owner: &T::AccountId) {
			let mut record = Self::record_by_id(id);
			let old_owner = record.owner.clone();
			let mut old_owner_records = Self::records_by_owner(old_owner.clone());
			let index = old_owner_records.iter().position(|value| value.id == *id).unwrap();
			old_owner_records.remove(index);
			RecordsByOwner::<T>::insert(old_owner, old_owner_records);
			let mut new_owner_records = Self::records_by_owner(new_owner);
			new_owner_records.push(record.clone());
			RecordsByOwner::<T>::insert(new_owner, new_owner_records);
			RecordById::<T>::insert(id, record);
		}

		fn update_record(record: &Record<T::AccountId>, id: &u128, signer: &T::AccountId) {
			RecordById::<T>::insert(id, record);
			let mut records = Self::records_by_owner(signer);
			records.push(record.clone());
			RecordsByOwner::<T>::insert(signer, records);
		}
	}
}