multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use tfn_dex::common::errors::*;
use tfn_platform::common::errors::*;
use tfn_platform::common::config::ProxyTrait as _;

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
        require!(!self.platform_sc().is_empty(), ERROR_PLATFORM_NOT_SET);
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

    #[only_owner]
    #[endpoint(setPlatformAddress)]
    fn set_platform_address(&self, platform_sc: ManagedAddress) {
        require!(self.platform_sc().is_empty(), ERROR_PLATFORM_ALREADY_SET);

        self.platform_sc().set(&platform_sc);
        let governance_token = self.platform_contract_proxy()
            .contract(platform_sc)
            .governance_token()
            .execute_on_dest_context::<TokenIdentifier>();
        self.base_tokens().insert(governance_token);
    }

    // base tokens
    #[storage_mapper("base_tokens")]
    fn base_tokens(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(getBaseTokens)]
    fn get_base_tokens(&self) -> ManagedVec<TokenIdentifier<Self::Api>> {
        let mut base_tokens = ManagedVec::new();
        for token in self.base_tokens().iter() {
            base_tokens.push(token);
        }

        base_tokens
    }

    // pairs
    #[view(getPair)]
    #[storage_mapper("pairs")]
    fn pair(&self, id: usize) -> SingleValueMapper<Pair<Self::Api>>;

    #[view(getLastPairId)]
    #[storage_mapper("last_pair_id")]
    fn last_pair_id(&self) -> SingleValueMapper<usize>;

    #[view(getPairs)]
    fn get_pairs(&self) -> ManagedVec<Pair<Self::Api>> {
        let mut pairs = ManagedVec::new();
        for id in 0..self.last_pair_id().get() {
            pairs.push(self.pair(id).get());
        }

        pairs
    }

    #[view(getPairByTickers)]
    fn get_pair_by_tickers(&self, token1: &TokenIdentifier, token2: &TokenIdentifier) -> Option<Pair<Self::Api>> {
        for id in 0..self.last_pair_id().get() {
            let pair = self.pair(id).get();
            if &pair.base_token == token1 && &pair.token == token2 {
                return Some(pair);
            }
            if &pair.token == token1 && &pair.base_token == token2 {
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

    // proxies
    #[proxy]
    fn platform_contract_proxy(&self) -> tfn_platform::Proxy<Self::Api>;
}
