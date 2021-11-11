#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;
	use codec::{Encode, Decode};

	#[derive(Debug, Clone, PartialEq, Encode, Decode, Default, scale_info::TypeInfo)]
	pub struct Post<AccountId, Comments> {
		author: AccountId,
		id: u128,
		likes: u32,
		date: Vec<u8>,
		handle_tags: Vec<u128>,
		hashtags: Vec<u128>,
		content: Vec<u8>,
		comments: Vec<Comments>,
		total_comments: u32,
		images: Vec<Vec<u8>>,
	}

	#[derive(Debug, Clone, PartialEq, Encode, Decode, Default, scale_info::TypeInfo)]
	pub struct Comment<AccountId> {
		author: AccountId,
		post_id: u128,
		comment: Vec<u8>,
		likes: u32,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_users::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn get_user_posts)]
	pub(super) type Posts<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<Post<T::AccountId, Comment<T::AccountId>>>, ValueQuery>;

	#[pallet::storage]
	pub type PostByCount<T: Config> = StorageMap<_, Twox64Concat, u128, Post<T::AccountId, Comment<T::AccountId>>>;

	#[pallet::storage]
	#[pallet::getter(fn get_comments_for_post)]
	pub(super) type CommentsByPost<T: Config> = StorageDoubleMap<_, Twox64Concat, u128, Twox64Concat, T::AccountId, Vec<Comment<T::AccountId>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_comment_by_id)]
	pub(super) type CommentsById<T: Config> = StorageMap<_, Twox64Concat, u128, Comment<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	pub type CommentsCount<T> = StorageValue<_, u128>;

	#[pallet::storage]
	#[pallet::getter(fn get_user_following)]
	pub(super) type Following<T: Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u128, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_user_following)]
	pub(super) type IsFollowing<T: Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_user_followers)]
	pub(super) type Followers<T: Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, u128, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn is_user_follower)]
	pub(super) type IsFollower<T: Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	pub type PostCount<T: Config> = StorageValue<_, u128>;

	#[pallet::storage]
	#[pallet::getter(fn has_user_liked)]
	pub(super) type HasLiked<T: Config> = StorageDoubleMap<_, Twox64Concat, u128, Twox64Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	pub type HashtagPosts<T: Config> = StorageMap<_, Twox64Concat ,u128, Vec<Post<T::AccountId, Comment<T::AccountId>>>>;

	#[pallet::storage]
	#[pallet::getter(fn liked_by)]
	pub(super) type LikedBy<T: Config> = StorageDoubleMap<_, Twox64Concat, u128, Twox64Concat, T::AccountId, (Vec<u8>, T::AccountId), ValueQuery>;

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
		AlreadyLiked
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn new_post(
			origin: OriginFor<T>,
			date: Vec<u8>,
			handle_tags: Vec<u128>,
			hashtags: Vec<u128>,
			content: Vec<u8>,
			images: Vec<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let count = PostCount::<T>::get().unwrap_or(0);
			PostByCount::<T>::insert(count.clone(), Post {
				author: who.clone(),
				id: count.clone(),
				likes: 0,
				date,
				handle_tags,
				hashtags: hashtags.clone(),
				content,
				comments: Vec::new(),
				total_comments: 0,
				images,
			});
			let post = PostByCount::<T>::get(count.clone()).unwrap_or(Default::default());
			Self::add_to_user_posts(&post, &who);
			Self::add_to_hashtag_posts(&hashtags, &post);
			PostCount::<T>::put(count + 1);
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn new_comment(
			origin: OriginFor<T>,
			post_id: u128,
			comment: Vec<u8>,
			author: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let count = CommentsCount::<T>::get().unwrap_or(0);
			CommentsById::<T>::insert(count.clone(), Comment {
				author: who,
				post_id,
				comment,
				likes: 0,
			});
			let comment = CommentsById::<T>::get(count.clone());
			Self::add_to_post_comments(&post_id, &comment, &author);
			CommentsCount::<T>::put(count + 1);
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn like_post(
			origin: OriginFor<T>,
			post_id: u128,
			author: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Self::has_user_liked(post_id.clone(), who.clone()), Error::<T>::AlreadyLiked);
			let user = pallet_users::Pallet::<T>::get_user(&who);
			LikedBy::<T>::insert(post_id.clone(), who.clone(), (user.handle, user.address));
			let mut post = PostByCount::<T>::get(post_id.clone()).unwrap_or(Default::default());
			post.likes = post.likes + 1;
			Self::update_user_posts_likes(&post_id, &author);
			PostByCount::<T>::insert(post_id.clone(), post);
			HasLiked::<T>::insert(post_id, who, true);
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn follow(
			origin: OriginFor<T>,
			user_handle_id: u128,
			user_to_follow: T::AccountId,
			user_to_follow_handle_id: u128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::add_to_followers(&user_to_follow, &user_to_follow_handle_id, &who);
			Self::add_to_following(&who, &user_handle_id, &user_to_follow);
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn unfollow(
			origin: OriginFor<T>,
			user_to_unfollow: T::AccountId,
			user_to_unfollow_handle_id: u128,
			user_handle_id: u128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::remove_followers(&user_to_unfollow, &user_to_unfollow_handle_id, &who);
			Self::remove_following(&user_to_unfollow, &user_to_unfollow_handle_id, &who);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		
		fn remove_followers(
			user_to_unfollow: &T::AccountId, 
			user_to_unfollow_handle_id: &u128,
			user_initiating_unfollow: &T::AccountId,
		) {
			let mut followers = Followers::<T>::get(user_to_unfollow, user_to_unfollow_handle_id);
			let index = followers.iter().position(|value| value == user_initiating_unfollow).unwrap();
			followers.remove(index);
			IsFollower::<T>::insert(user_initiating_unfollow, user_to_unfollow, false);
			Followers::<T>::insert(user_to_unfollow, user_to_unfollow_handle_id, followers);
		}

		fn remove_following(
			user_to_unfollow: &T::AccountId, 
			user_initiating_unfollow_handle_id: &u128,
			user_initiating_unfollow: &T::AccountId,
		) {
			let mut following = Following::<T>::get(user_initiating_unfollow, user_initiating_unfollow_handle_id);
			let index = following.iter().position(|value| value == user_to_unfollow).unwrap();
			following.remove(index);
			IsFollowing::<T>::insert(user_initiating_unfollow, user_to_unfollow, false);
			Following::<T>::insert(user_initiating_unfollow, user_initiating_unfollow_handle_id, following);
		}

		fn add_to_followers(
			user_to_follow: &T::AccountId, 
			user_to_follow_handle_id: &u128,
			user_initiating_follow: &T::AccountId,
		) {
			let mut followers = Followers::<T>::get(user_to_follow, user_to_follow_handle_id);
			followers.push(user_initiating_follow.clone());
			IsFollower::<T>::insert(user_initiating_follow, user_to_follow, true);
			Followers::<T>::insert(user_to_follow, user_to_follow_handle_id, followers);
		}

		fn add_to_following(
			user_initiating_follow: &T::AccountId, 
			user_initiating_follow_handle_id: &u128,
			user_to_follow: &T::AccountId,
		) {
			let mut following = Following::<T>::get(user_initiating_follow, user_initiating_follow_handle_id);
			following.push(user_to_follow.clone());
			IsFollowing::<T>::insert(user_to_follow, user_initiating_follow, true);
			Following::<T>::insert(user_initiating_follow, user_initiating_follow_handle_id, following);
		}

		fn add_to_hashtag_posts(
			ht: &Vec<u128>, 
			post: &Post<T::AccountId, Comment<T::AccountId>>
		) {
			for h in ht {
				let mut ht_posts = HashtagPosts::<T>::get(h).unwrap_or(Default::default());
				let post_c = post.clone();
				ht_posts.push(post_c);
				HashtagPosts::<T>::insert(h, ht_posts);
			}
		}

		fn add_to_post_comments(
			post_id: &u128, 
			comment: &Comment<T::AccountId>, 
			author: &T::AccountId
		) {
			let mut post = PostByCount::<T>::get(post_id).unwrap();
			post.total_comments += 1;
			post.comments.push(comment.clone());
			PostByCount::<T>::insert(post_id, post);
			let mut author_posts = Posts::<T>::get(author);
			let index = author_posts.iter().position(|value| value.id == *post_id).unwrap();
			author_posts[index].total_comments += 1;
			author_posts[index].comments.push(comment.clone());
			let mut current_comments = CommentsByPost::<T>::get(post_id, author);
			current_comments.push(comment.clone());
			CommentsByPost::<T>::insert(post_id, author, current_comments);
		}

		fn add_to_user_posts(
			post: &Post<T::AccountId, Comment<T::AccountId>>, 
			user: &T::AccountId
		) {
			let mut posts = Posts::<T>::get(user);
			let post_c = post.clone();
			posts.push(post_c);
			Posts::<T>::insert(user, posts);
		}

		fn update_user_posts_likes(
			id: &u128, 
			author: &T::AccountId
		) {
			let mut posts = Posts::<T>::get(author);
			let index = posts.iter().position(|value| value.id == *id).unwrap();
			posts[index].likes += 1;
			Posts::<T>::insert(author, posts);
		}

		fn check_is_user(id: &T::AccountId) -> bool {
			let user = pallet_users::Pallet::<T>::get_user(id);
			if user.address.eq(id) {
				true
			} else {
				false
			}
		}
	}
}
