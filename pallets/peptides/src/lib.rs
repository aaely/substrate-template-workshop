#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;
	use codec::{Encode, Decode};

	#[derive(Debug, Clone, PartialEq, Default, Encode, Decode, scale_info::TypeInfo)]
	pub struct Peptide<AccountId> {
		created_by: AccountId,
		id: u128,
		name: Vec<u8>,
		price: u32,
		inventory: u32,
		image_hash: Vec<u8>,
	}

	#[derive(Debug, Clone, PartialEq, Default, Encode, Decode, scale_info::TypeInfo)]
	pub struct PeptideProfile<AminoAcid> {
		peptide_ref: u128,
		chain: Vec<AminoAcid>,
		production_cost: u32,
		production_yield: u32,
	}

	#[derive(Debug, Clone, PartialEq, Default, Encode, Decode, scale_info::TypeInfo)]
	pub struct AminoAcid {
		id: u128,
		name: Vec<u8>,
		cost: u32,
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
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	/// Gets a peptide stored by sequential u128 id
	#[pallet::storage]
	#[pallet::getter(fn get_peptide)]
	pub(super) type Peptides<T: Config> = StorageMap<_, Twox64Concat, u128, (Peptide<T::AccountId>, PeptideProfile<AminoAcid>), ValueQuery>;

	/// Gets an amino acid stored by sequential u128 id
	#[pallet::storage]
	#[pallet::getter(fn get_amino)]
	pub(super) type AminoAcids<T> = StorageMap<_, Twox64Concat, u128, AminoAcid, ValueQuery>;

	#[pallet::storage]
	pub type PeptideByCount<T: Config> = StorageMap<_, Twox64Concat, u32, (Peptide<T::AccountId>, PeptideProfile<AminoAcid>)>;

	#[pallet::storage]
	pub(super) type AminoAcidByCount<T> = StorageMap<_, Twox64Concat, u32, AminoAcid>;

	#[pallet::storage]
	pub type PeptideCount<T: Config> = StorageValue<_, u32>;

	#[pallet::storage]
	pub(super) type AminoAcidCount<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		NewAmino(AminoAcid),
		NewPeptide((Peptide<T::AccountId>, PeptideProfile<AminoAcid>)),
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

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// Exposed extrinsic use to create a new peptide. Prevents duplicated from being added.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2))]
		pub fn create_peptide(
			origin: OriginFor<T>, 
			name: Vec<u8>, 
			id: u128, 
			price: u32, 
			inventory: u32, 
			image_hash: Vec<u8>,
			chain: Vec<AminoAcid>) -> DispatchResult {
				let who = ensure_signed(origin)?;
				ensure!(!Self::check_duplicate_peptide(&id), Error::<T>::ItemAlreadyExists);
				let count = PeptideCount::<T>::get().unwrap_or(0);
				let production_cost = Self::production_cost_calc(&chain).0;
				let production_yield = Self::production_cost_calc(&chain).1;
				// Update storage.
				Peptides::<T>::insert(id.clone(), (Peptide {
					created_by: who,
					id: id.clone(),
					name,
					price,
					inventory,
					image_hash
				}, PeptideProfile {
					peptide_ref: id.clone(),
					chain,
					production_cost,
					production_yield,
				}));
				let peptide = Peptides::<T>::get(id);
				Self::add_peptide_by_count(&count, &peptide);
				Self::deposit_event(Event::NewPeptide(peptide));
				PeptideCount::<T>::put(count + 1);				
				Ok(())
			}

		/// Exposed extrinsic used to create a new amino acid. 
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,3))]
		pub fn create_amino(
			origin: OriginFor<T>, 
			name: Vec<u8>, 
			id: u128, 
			cost: u32) -> DispatchResult {
				let who = ensure_signed(origin)?;
				ensure!(!Self::check_duplicate_amino(&id), Error::<T>::ItemAlreadyExists);
				let count = AminoAcidCount::<T>::get().unwrap_or(0);
				AminoAcids::<T>::insert(id.clone(), AminoAcid {
					id: id.clone(),
					name,
					cost,
				});
				let amino = AminoAcids::<T>::get(id);
				Self::add_amino_by_count(&count, &amino);
				Self::deposit_event(Event::NewAmino(amino));
				AminoAcidCount::<T>::put(count + 1);
				Ok(())
			}
	}

	impl<T: Config> Pallet<T> {
		
		fn check_duplicate_peptide(id: &u128) -> bool {
			let peptide = Self::get_peptide(id);
			if peptide.0.name.len() > 0 {
				true
			} else {
				false
			}
		}

		fn check_duplicate_amino(id: &u128) -> bool {
			let amino = Self::get_amino(id);
			if amino.name.len() > 0 {
				true
			} else {
				false
			}
		}

		fn add_peptide_by_count(
			count: &u32, 
			peptide: &(Peptide<T::AccountId>, PeptideProfile<AminoAcid>)
			) {
				PeptideByCount::<T>::insert(count, peptide);
		}

		fn add_amino_by_count(
			count: &u32,
			amino: &AminoAcid
		) {
			AminoAcidByCount::<T>::insert(count, amino)
		}

		fn production_cost_calc(amino_chain: &Vec<AminoAcid>) -> (u32, u32) {
			///
			/// I had entirely too many issues with .pow() so I ended up doing this instead
			/// calculates the theoretic yield of the supplied amino chain
			let mut total: u32 = 0;
			let mut yld: f64 = 0.97;
			for amino in amino_chain {
				total += total + amino.cost;
				yld = yld * 0.97
			}
			yld = yld / 0.97;
			yld = yld * 100.0;
			(total, yld as u32)
		}
	}
}