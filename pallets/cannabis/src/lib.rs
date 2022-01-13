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

	#[derive(Debug, Clone, PartialEq, Encode, Decode, scale_info::TypeInfo)]
	pub struct CannabisProduct {
		id: u128,
		name: Vec<u8>,
		price: u32,
		category: CannabisCategory,
		inventory: u32,
		image_hash: Vec<u8>,
		cannabinoids: Vec<(u128, Vec<u8>, u32)>,
		terpenes: Vec<(u128, Vec<u8>, u32)>,
	}

	impl Default for CannabisProduct {
		fn default() -> Self {
			CannabisProduct {
				id: Default::default(),
				name: Default::default(),
				price: Default::default(),
				category: CannabisCategory::Flower,
				inventory: Default::default(),
				image_hash: Default::default(),
				cannabinoids: Default::default(),
				terpenes: Default::default(),
			}

		}
	}

	#[derive(Debug, Clone, PartialEq, Default, Encode, Decode, scale_info::TypeInfo)]
	pub struct Cannabinoid {
		id: u128,
		name: Vec<u8>,
		description: Vec<u8>,
		products: Vec<(u128, u32)>,
	}

	#[derive(Debug, Clone, PartialEq, Default, Encode, Decode, scale_info::TypeInfo)]
	pub struct Terpene {
		id: u128,
		name: Vec<u8>,
		description: Vec<u8>,
		products: Vec<(u128, u32)>,
	}

	#[derive(Debug, Clone, PartialEq, Encode, Decode, scale_info::TypeInfo)]
	pub enum CannabisCategory {
		Flower,
		CO2Extract,
		ButaneExtract,
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
	#[pallet::getter(fn get_cannabis_product)]
	pub (super) type CannabisProducts<T> = StorageMap<_, Twox64Concat, u128, CannabisProduct>;

	#[pallet::storage]
	pub type CannabisCount<T: Config> = StorageValue<_, u32>;

	#[pallet::storage]
	pub type CannabisProductByCount<T> = StorageMap<_, Twox64Concat, u32, CannabisProduct>;

	#[pallet::storage]
	pub type CannabisProductsByCategory<T> = StorageMap<_, Twox64Concat, CannabisCategory, Vec<CannabisProduct>>;

	#[pallet::storage]
	#[pallet::getter(fn get_terpene)]
	pub (super) type Terpenes<T> = StorageMap<_, Twox64Concat, u128, Terpene>;

	#[pallet::storage]
	#[pallet::getter(fn get_cannabinoid)]
	pub (super) type Cannabinoids<T> = StorageMap<_, Twox64Concat, u128, Cannabinoid>;

	#[pallet::storage]
	pub type TerpeneByCount<T> = StorageMap<_, Twox64Concat, u32, Terpene>;

	#[pallet::storage]
	pub type CannabinoidByCount<T> = StorageMap<_, Twox64Concat, u32, Cannabinoid>;

	#[pallet::storage]
	pub type TerpeneCount<T> = StorageValue<_, u32>;

	#[pallet::storage]
	pub type CannabinoidCount<T> = StorageValue<_, u32>;


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
		ItemAlreadyExists,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Exposed extrinsic for creating a new cannabis product. 
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
		pub fn new_cannabis_product(
			origin: OriginFor<T>, 
			id: u128, 
			name: Vec<u8>, 
			price: u32, 
			category: CannabisCategory, 
			inventory: u32,
			image_hash: Vec<u8>, 
			cannabinoids: Vec<(u128, Vec<u8>, u32)>, 
			terpenes: Vec<(u128, Vec<u8>, u32)>) -> DispatchResult {
				ensure_signed(origin)?;
				ensure!(!Self::check_duplicate_cannabis(&id), Error::<T>::ItemAlreadyExists);
				let count = CannabisCount::<T>::get().unwrap_or(0);
				Self::add_product_to_cannabinoid(&id, &cannabinoids);
				Self::add_product_to_terpene(&id, &terpenes);
				CannabisProducts::<T>::insert(id.clone(), CannabisProduct {
					id: id.clone(),
					name,
					price,
					category,
					inventory,
					image_hash,
					cannabinoids,
					terpenes,
				});
				let product = CannabisProducts::<T>::get(id).unwrap_or(Default::default());
				Self::add_cannabis_product_by_count(&count, &product);
				match product.category {
					CannabisCategory::Flower => {
						let mut products = CannabisProductsByCategory::<T>::get(CannabisCategory::Flower).unwrap_or(Default::default());
						products.push(product);
						CannabisProductsByCategory::<T>::insert(CannabisCategory::Flower, products);
					},
					CannabisCategory::CO2Extract => {
						let mut products = CannabisProductsByCategory::<T>::get(CannabisCategory::CO2Extract).unwrap_or(Default::default());
						products.push(product);
						CannabisProductsByCategory::<T>::insert(CannabisCategory::CO2Extract, products);
					},
					CannabisCategory::ButaneExtract => {
						let mut products = CannabisProductsByCategory::<T>::get(CannabisCategory::ButaneExtract).unwrap_or(Default::default());
						products.push(product);
						CannabisProductsByCategory::<T>::insert(CannabisCategory::ButaneExtract, products);
					},
				}
				CannabisCount::<T>::put(count + 1);
				Ok(())
		}
		
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
		pub fn new_terpene(
			origin: OriginFor<T>, 
			id: u128, 
			name: Vec<u8>, 
			description: Vec<u8>) -> DispatchResult {
				ensure_signed(origin)?;
				//ensure!(Self::get_admin(who.clone()), Error::<T>::InsufficientPriv);
				ensure!(!Self::check_duplicate_terpene(&id), Error::<T>::ItemAlreadyExists);
				let count = TerpeneCount::<T>::get().unwrap_or(0);
				Terpenes::<T>::insert(id.clone(), Terpene {
					id: id.clone(),
					name,
					description,
					products: Vec::new(),
				});
				let terp = Terpenes::<T>::get(id).unwrap_or(Default::default());
				Self::add_terpene_by_count(&count, &terp);
				TerpeneCount::<T>::put(count + 1);
				Ok(())
		}
		
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
		pub fn new_cannabinoid(
			origin: OriginFor<T>, 
			id: u128, 
			name: Vec<u8>, 
			description: Vec<u8>) -> DispatchResult {
				ensure_signed(origin)?;
				ensure!(!Self::check_duplicate_cannabinoid(&id), Error::<T>::ItemAlreadyExists);
				let count = CannabinoidCount::<T>::get().unwrap_or(0);
				Cannabinoids::<T>::insert(id.clone(), Cannabinoid {
					id: id.clone(),
					name,
					description,
					products: Vec::new(),
				});
				let cann = Cannabinoids::<T>::get(id).unwrap_or(Default::default());
				Self::add_cannabinoid_by_count(&count, &cann);
				CannabinoidCount::<T>::put(count + 1);
				Ok(())
		}
	}

	impl<T: Config> Pallet<T> {

		fn add_cannabis_product_by_count(count: &u32, product: &CannabisProduct) {
			CannabisProductByCount::<T>::insert(count, product);
		}

		fn add_terpene_by_count(count: &u32, terpene: &Terpene) {
			TerpeneByCount::<T>::insert(count, terpene);
		}

		fn add_cannabinoid_by_count(count: &u32, cannabinoid: &Cannabinoid) {
			CannabinoidByCount::<T>::insert(count, cannabinoid);
		}

		/// Helper method to add a particular product to a particular terpene
		/// Used to quickly sort based on product terpene concentration
		fn add_product_to_terpene(id: &u128, terpenes: &Vec<(u128, Vec<u8>, u32)>) {
			for t in terpenes {
				let mut terp = Terpenes::<T>::get(t.0).unwrap_or(Default::default());
				terp.products.push((*id, t.2));
				Terpenes::<T>::insert(t.0, terp);
			}
		}

		/// Helper method to add a particular product to a particular cannabinoid
		/// Used to quickly sort based on product cannabinoid concentration
		fn add_product_to_cannabinoid(id: &u128, cannabinoids: &Vec<(u128, Vec<u8>, u32)>) {
			for c in cannabinoids {
				let mut cann = Cannabinoids::<T>::get(c.0).unwrap_or(Default::default());
				cann.products.push((*id, c.2));
				Cannabinoids::<T>::insert(c.0, cann);
			}
		}

		fn check_duplicate_terpene(id: &u128) -> bool {
			let terpene = Self::get_terpene(id).unwrap_or(Default::default());
			if terpene.name.len() > 0 {
				true
			} else {
				false
			}
		}

		fn check_duplicate_cannabinoid(id: &u128) -> bool {
			let cannabinoid = Self::get_cannabinoid(id).unwrap_or(Default::default());
			if cannabinoid.name.len() > 0 {
				true
			} else {
				false
			}
		}

		fn check_duplicate_cannabis(id: &u128) -> bool {
			let cannabis = Self::get_cannabis_product(id).unwrap_or(Default::default());
			if cannabis.name.len() > 0 {
				true
			} else {
				false
			}
		}
	}
}
