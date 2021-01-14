#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};

use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, Parameter, StorageValue, StorageMap, traits::Randomness};
use frame_system::ensure_signed;
use sp_io::hashing::blake2_128;
use sp_runtime::{DispatchError, traits::{AtLeast32BitUnsigned, Bounded, Member, MaybeSerialize, MaybeDisplay}};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode)]
pub struct Kitty (pub [u8; 16]);

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Randomness: Randomness<Self::Hash>;

    type KittyIndex: Parameter + Member + MaybeSerialize + Default + MaybeDisplay + AtLeast32BitUnsigned + Copy;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	trait Store for Module<T: Trait> as KittiesModule {
		pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;
		pub KittiesCount get(fn kitties_count): T::KittyIndex;
		pub KittiesOwners get(fn kitties_owners): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;

		pub OwnerKitties get(fn owner_kitties): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) T::KittyIndex => Option<()>;

        pub KittiesParents get(fn kitties_parents): map hasher(blake2_128_concat) T::KittyIndex => Option<(T::KittyIndex, T::KittyIndex)>;
		pub KittiesChildren get(fn kitties_children): double_map hasher(blake2_128_concat) T::KittyIndex, hasher(blake2_128_concat) T::KittyIndex => Option<()>;
		pub KittiesBreed get(fn kitties_breed): double_map hasher(blake2_128_concat) T::KittyIndex, hasher(blake2_128_concat) T::KittyIndex => Option<()>;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T>
	    where AccountId = <T as frame_system::Trait>::AccountId,
	    KittyIndex = <T as Trait>::KittyIndex
	{
		Create(AccountId, KittyIndex),
		Transfer(AccountId, AccountId, KittyIndex),
		Breed(AccountId, KittyIndex, KittyIndex, KittyIndex),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		///
		KittiesCountOverflow,
		///
		KittyNotOwn,
		InvalidKittyId,
		RequireDifferentParent,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

        #[weight = 0]
		pub fn create(origin) {
		    let sender = ensure_signed(origin)?;
		    let kitty_id = Self::next_kitty_id()?;
		    let dna = Self::random_value(&sender);
		    let kitty = Kitty(dna);

		    Self::insert_kitty(&sender, kitty_id, kitty);
		    Self::deposit_event(RawEvent::Create(sender, kitty_id));
		}

        #[weight = 0]
		pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex) {
		    let sender = ensure_signed(origin)?;
		    let owner = Self::kitties_owners(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;

		    ensure!(owner == sender, Error::<T>::KittyNotOwn);
		    <KittiesOwners<T>>::insert(kitty_id, to.clone());

            <OwnerKitties<T>>::remove(&sender, kitty_id);
            <OwnerKitties<T>>::insert(&to, kitty_id, ());

		    Self::deposit_event(RawEvent::Transfer(sender, to, kitty_id));
		}

        #[weight = 0]
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
		    let sender = ensure_signed(origin)?;
		    let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;

		    Self::deposit_event(RawEvent::Breed(sender, new_kitty_id, kitty_id_1, kitty_id_2));
		}
	}
}


fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    (selector & dna1) | (!selector & dna2)
}


impl <T: Trait> Module<T> {
    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        let kitty_id = Self::kitties_count();
        if kitty_id == T::KittyIndex::max_value() {
            return Err(Error::<T>::KittiesCountOverflow.into());
        }
        Ok(kitty_id)
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random_seed(),
            &sender,
            <frame_system::Module<T>>::extrinsic_index(),
        );
        payload.using_encoded(blake2_128)
    }

    fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
        let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

        ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);

        let kitty_id = Self::next_kitty_id()?;
        let kitty1_dna = kitty1.0;
        let kitty2_dna = kitty2.0;
        let selector = Self::random_value(&sender);
        let mut new_dna = [0u8; 16];

        for i in 0..kitty1_dna.len() {
            new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[2], selector[i]);
        }
        Self::insert_kitty(&sender, kitty_id, Kitty(new_dna));

        <OwnerKitties<T>>::insert(&sender, kitty_id, ());

        <KittiesParents<T>>::insert(kitty_id, (kitty_id_1, kitty_id_2));
        <KittiesChildren<T>>::insert(kitty_id_1, kitty_id, ());
        <KittiesChildren<T>>::insert(kitty_id_2, kitty_id, ());
        <KittiesBreed<T>>::insert(kitty_id_1, kitty_id_2, ());
        <KittiesBreed<T>>::insert(kitty_id_2, kitty_id_1, ());

        Ok(kitty_id)
    }

    fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
        <Kitties<T>>::insert(kitty_id, kitty);
        <KittiesCount<T>>::put(kitty_id + 1.into());
        <KittiesOwners<T>>::insert(kitty_id, owner.clone());

        <OwnerKitties<T>>::insert(owner, kitty_id, ());
    }

}
