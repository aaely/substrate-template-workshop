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
	use frame_system::{ensure_signed, pallet_prelude::*};
	use sp_std::prelude::*;
	use codec::{Encode, Decode};

	#[derive(Debug, Clone, PartialEq, Encode, Decode, Default, scale_info::TypeInfo)]
	pub struct Post<AccountId> {
		author: AccountId,
		id: u128,
		likes: u32,
		date: Vec<u8>,
		handle_tags: Vec<u128>,
		hashtags: Vec<u128>,
		content: Vec<u8>,
		comments: Vec<u128>,
		total_comments: u32,
		images: Vec<Vec<u8>>,
	}

	#[derive(Debug, Clone, PartialEq, Encode, Decode, Default, scale_info::TypeInfo)]
	pub struct Comment<AccountId> {
		author: AccountId,
		post_id: u128,
		comment_id: u128,
		comment: Vec<u8>,
		likes: u32,
		date: Vec<u8>,
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
	pub(super) type AccountAllPosts<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<Post<T::AccountId>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_post_by_id)]
	pub(super) type PostById<T: Config> = StorageMap<_, Twox64Concat, u128, Post<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_comment_by_id)]
	pub(super) type CommentById<T: Config> = StorageMap<_, Twox64Concat, u128, Comment<T::AccountId>, ValueQuery>;

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
	#[pallet::getter(fn post_has_user_liked)]
	//post_id -> u128 || user_who_liked -> T::AccountId
	pub(super) type HasLikedPost<T: Config> = StorageDoubleMap<_, Twox64Concat, u128, Twox64Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn comment_has_user_liked)]
	//comment_id -> u128 || user_who_liked -> T::AccountId
	pub(super) type HasLikedComment<T: Config> = StorageDoubleMap<_, Twox64Concat, u128, Twox64Concat, T::AccountId, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_posts_by_hashtag_id)]
	pub(super) type HashtagPosts<T: Config> = StorageMap<_, Twox64Concat ,u128, Vec<Post<T::AccountId>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn post_liked_by)]
	//post_id -> u128 || user_who_liked -> T::AccountId
	pub(super) type PostLikedBy<T: Config> = StorageMap<_, Twox64Concat, u128, Vec<(Vec<u8>, T::AccountId)>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn comment_liked_by)]
	//comment_id -> u128 || user_who_liked -> T::AccountId
	pub(super) type CommentLikedBy<T: Config> = StorageMap<_, Twox64Concat, u128, Vec<(Vec<u8>, T::AccountId)>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		NewComment(Comment<T::AccountId>),
		NewPost(Post<T::AccountId>),
		PostLiked(u128),
		CommentLiked(u128, u128),
		PostUnliked(u128),
		CommentUnliked(u128, u128),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		AlreadyLiked,
		NotLikedYet,
		NotAuthor,
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
			PostById::<T>::insert(count.clone(), Post {
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
			let post = PostById::<T>::get(count.clone());
			Self::add_to_user_posts(&post, &who);
			Self::add_to_hashtag_posts(&hashtags, &post);
			PostCount::<T>::put(count + 1);
			Self::deposit_event(Event::<T>::NewPost(post));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn remove_post(
			origin: OriginFor<T>,

		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Self::check_is_user(&who), Error::<T>::NotAuthor);
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn new_comment(
			origin: OriginFor<T>,
			post_id: u128,
			comment: Vec<u8>,
			post_author: T::AccountId,
			date: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let count = CommentsCount::<T>::get().unwrap_or(0);
			CommentById::<T>::insert(count.clone(), Comment {
				author: who,
				post_id,
				comment_id: count.clone(),
				comment,
				likes: 0,
				date,
			});
			let comment = CommentById::<T>::get(count.clone());
			Self::add_to_post_comments(&post_id, &comment, &post_author);
			CommentsCount::<T>::put(count + 1);
			Self::deposit_event(Event::<T>::NewComment(comment));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn remove_comment(
			origin: OriginFor<T>,

		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Self::check_is_user(&who), Error::<T>::NotAuthor);
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn like_post(
			origin: OriginFor<T>,
			post_id: u128,
			author: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Self::post_has_user_liked(post_id.clone(), who.clone()), Error::<T>::AlreadyLiked);
			let mut post = PostById::<T>::get(post_id.clone());
			post.likes = post.likes + 1;
			PostById::<T>::insert(post_id.clone(), post);
			Self::update_user_posts_likes(&post_id, &author);
			Self::post_liked(&who, &post_id);
			Self::deposit_event(Event::<T>::PostLiked(post_id));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn unlike_post(
			origin: OriginFor<T>,
			post_id: u128,
			author: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::post_has_user_liked(post_id.clone(), who.clone()), Error::<T>::NotLikedYet);
			let mut post = PostById::<T>::get(post_id.clone());
			post.likes -= 1;
			PostById::<T>::insert(post_id.clone(), post);
			Self::update_user_posts_unlikes(&post_id, &author);
			Self::post_unliked_by(&who, &post_id);
			Self::deposit_event(Event::<T>::PostUnliked(post_id));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn like_comment(
			origin: OriginFor<T>,
			comment_id: u128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Self::comment_has_user_liked(comment_id.clone(), who.clone()), Error::<T>::AlreadyLiked);
			Self::comment_liked(&who, &comment_id);
			let mut comment = CommentById::<T>::get(comment_id);
			comment.likes += 1;
			CommentById::<T>::insert(comment_id, comment.clone());
			Self::deposit_event(Event::<T>::CommentLiked(comment.post_id.clone(), comment_id));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3,3))]
		pub fn unlike_comment(
			origin: OriginFor<T>,
			comment_id: u128,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(Self::comment_has_user_liked(comment_id.clone(), who.clone()), Error::<T>::NotLikedYet);
			Self::comment_unliked_by(&who, &comment_id);
			let mut comment = CommentById::<T>::get(comment_id);
			comment.likes -= 1;
			CommentById::<T>::insert(comment_id, comment.clone());
			Self::deposit_event(Event::<T>::CommentUnliked(comment.post_id.clone(), comment_id));
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
			Self::remove_following(&user_to_unfollow, &user_handle_id, &who);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {

		fn post_liked(
			user_liked: &T::AccountId, 
			post_id: &u128, 
		) {
			let user = pallet_users::Pallet::<T>::get_user(user_liked);
			let mut current_likes = PostLikedBy::<T>::get(post_id);
			current_likes.push((user.handle, user.address));
			PostLikedBy::<T>::insert(post_id, current_likes);
			HasLikedPost::<T>::insert(post_id, user_liked, true);
		}

		fn post_unliked_by(
			user_liked: &T::AccountId, 
			post_id: &u128, 
		) {
			let mut current_likes = PostLikedBy::<T>::get(post_id);
			let index = current_likes.iter().position(|value| value.1 == *user_liked).unwrap();
			current_likes.remove(index);
			PostLikedBy::<T>::insert(post_id, current_likes);
			HasLikedPost::<T>::insert(post_id, user_liked, false);
		}

		

		fn comment_liked(
			user_liked: &T::AccountId, 
			comment_id: &u128,
		) {
			let user = pallet_users::Pallet::<T>::get_user(user_liked);
			let mut current_likes = CommentLikedBy::<T>::get(comment_id);
			current_likes.push((user.handle, user.address));
			CommentLikedBy::<T>::insert(comment_id, current_likes);
			HasLikedComment::<T>::insert(comment_id, user_liked, true);
		}

		fn comment_unliked_by(
			user_unliked: &T::AccountId,
			comment_id: &u128,
		) {
			let mut current_likes = CommentLikedBy::<T>::get(comment_id);
			let index = current_likes.iter().position(|value| value.1 == *user_unliked).unwrap();
			current_likes.remove(index);
			CommentLikedBy::<T>::insert(comment_id, current_likes);
			HasLikedComment::<T>::insert(comment_id, user_unliked, false);
		}
		
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
			post: &Post<T::AccountId>
		) {
			for h in ht {
				let mut ht_posts = HashtagPosts::<T>::get(h);
				ht_posts.push(post.clone());
				HashtagPosts::<T>::insert(h, ht_posts);
			}
		}

		fn add_to_post_comments(
			post_id: &u128, 
			comment: &Comment<T::AccountId>, 
			post_author: &T::AccountId
		) {
			let mut post = PostById::<T>::get(post_id);
			post.total_comments += 1;
			post.comments.push(comment.comment_id.clone());
			PostById::<T>::insert(post_id, post);
			let mut author_posts = AccountAllPosts::<T>::get(post_author);
			let index = author_posts.iter().position(|value| value.id == *post_id).unwrap();
			author_posts[index].total_comments += 1;
			author_posts[index].comments.push(comment.comment_id.clone());
			AccountAllPosts::<T>::insert(post_author, author_posts);
		}

		fn add_to_user_posts(
			post: &Post<T::AccountId>, 
			user: &T::AccountId
		) {
			let mut posts = AccountAllPosts::<T>::get(user);
			let mut _user = pallet_users::Pallet::<T>::get_user(user);
			posts.push(post.clone());
			AccountAllPosts::<T>::insert(user, posts);
			_user.total_posts += 1;
			pallet_users::Pallet::<T>::insert_user(user, &_user);
		}

		fn update_user_posts_likes(
			id: &u128, 
			author: &T::AccountId
		) {
			let mut posts = AccountAllPosts::<T>::get(author);
			let index = posts.iter().position(|value| value.id == *id).unwrap();
			posts[index].likes += 1;
			PostById::<T>::insert(id, posts[index].clone());
			
			AccountAllPosts::<T>::insert(author, posts);
		}

		fn update_user_posts_unlikes(
			id: &u128, 
			author: &T::AccountId
		) {
			let mut posts = AccountAllPosts::<T>::get(author);
			let index = posts.iter().position(|value| value.id == *id).unwrap();
			posts[index].likes -= 1;
			PostById::<T>::insert(id, posts[index].clone());
			AccountAllPosts::<T>::insert(author, posts);
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
