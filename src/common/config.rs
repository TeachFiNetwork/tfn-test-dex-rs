multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use tfn_dex::common::errors::*;

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum PairState {
    Inactive,
    ActiveNoSwap,
    Active,
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Clone, Debug)]
pub struct Pair<M: ManagedTypeApi> {
    pub id: usize,
    pub owner: ManagedAddress<M>,
    pub state: PairState,
    pub token: TokenIdentifier<M>,
    pub decimals: u8,
    pub base_token: TokenIdentifier<M>,
    pub lp_token: TokenIdentifier<M>,
    pub lp_supply: BigUint<M>,
    pub lp_fee: u64,
    pub owner_fee: u64,
    pub liquidity_token: BigUint<M>,
    pub liquidity_base: BigUint<M>,
}

#[multiversx_sc::module]
pub trait ConfigModule {
    // state
    #[only_owner]
    #[endpoint(setStateActive)]
    fn set_state_active(&self) {
        require!(!self.base_tokens().is_empty(), ERROR_NO_BASE_TOKENS);

        self.state().set(State::Active);
    }

    #[only_owner]
    #[endpoint(setStateInactive)]
    fn set_state_inactive(&self) {
        self.state().set(State::Inactive);
    }

    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;

    // platform sc address
    #[view(getPlatformAddress)]
    #[storage_mapper("platform_address")]
    fn platform_sc(&self) -> SingleValueMapper<ManagedAddress>;

    // base tokens
    #[view(getBaseTokens)]
    #[storage_mapper("base_tokens")]
    fn base_tokens(&self) -> UnorderedSetMapper<TokenIdentifier>;

    // pairs
    #[view(getPair)]
    #[storage_mapper("pairs")]
    fn pair(&self, id: usize) -> SingleValueMapper<Pair<Self::Api>>;

    #[view(getLastPairId)]
    #[storage_mapper("last_pair_id")]
    fn last_pair_id(&self) -> SingleValueMapper<usize>;

    #[view(getPairs)]
    fn get_pairs(&self) -> ManagedVec<Pair<Self::Api>> {
        let last_pair_id = self.last_pair_id().get();
        let mut pairs = ManagedVec::new();
        for id in 0..last_pair_id {
            pairs.push(self.pair(id).get());
        }

        pairs
    }

    #[view(getPairByTickers)]
    fn get_pair_by_tickers(&self, base_token: &TokenIdentifier, token: &TokenIdentifier) -> Option<Pair<Self::Api>> {
        let last_pair_id = self.last_pair_id().get();
        for id in 0..last_pair_id {
            let pair = self.pair(id).get();
            if &pair.base_token == base_token && &pair.token == token {
                return Some(pair);
            }
            if &pair.token == base_token && &pair.base_token == token {
                return Some(pair);
            }
        }

        None
    }

    #[view(getPairByLpToken)]
    fn get_pair_by_lp_token(&self, lp_token: &TokenIdentifier) -> Option<Pair<Self::Api>> {
        let last_pair_id = self.last_pair_id().get();
        for id in 0..last_pair_id {
            let pair = self.pair(id).get();
            if &pair.lp_token == lp_token {
                return Some(pair);
            }
        }

        None
    }
}
